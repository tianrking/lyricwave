import Foundation
import AVFoundation
import ScreenCaptureKit
import CoreMedia

struct Selector {
    var pid: Int32?
    var nameContains: String?
}

enum Mode {
    case listApps
    case capture(CaptureArgs)
}

struct CaptureArgs {
    var outputURL: URL
    var sampleRate: UInt32
    var channels: UInt16
    var seconds: UInt32?
    var selectors: [Selector]
}

final class AudioCaptureOutput: NSObject, SCStreamOutput {
    let writer: AVAssetWriter
    let writerInput: AVAssetWriterInput
    private var started = false

    init(outputURL: URL) throws {
        writer = try AVAssetWriter(outputURL: outputURL, fileType: .m4a)
        writerInput = AVAssetWriterInput(mediaType: .audio, outputSettings: [
            AVFormatIDKey: kAudioFormatMPEG4AAC,
            AVSampleRateKey: 48000,
            AVNumberOfChannelsKey: 2,
        ])
        writerInput.expectsMediaDataInRealTime = true
        if writer.canAdd(writerInput) {
            writer.add(writerInput)
        } else {
            throw NSError(domain: "lyricwave.sc", code: 1, userInfo: [
                NSLocalizedDescriptionKey: "cannot add audio writer input",
            ])
        }
    }

    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of outputType: SCStreamOutputType) {
        guard outputType == .audio else { return }
        guard CMSampleBufferIsValid(sampleBuffer) else { return }

        if !started {
            started = true
            writer.startWriting()
            let pts = CMSampleBufferGetPresentationTimeStamp(sampleBuffer)
            writer.startSession(atSourceTime: pts)
        }

        if writerInput.isReadyForMoreMediaData {
            _ = writerInput.append(sampleBuffer)
        }
    }

    func finish() async throws {
        writerInput.markAsFinished()
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            writer.finishWriting {
                if let err = self.writer.error {
                    continuation.resume(throwing: err)
                } else {
                    continuation.resume(returning: ())
                }
            }
        }
    }
}

@main
struct Main {
    static func main() async {
        do {
            let mode = try parseArguments()
            let shareable = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: false)

            switch mode {
            case .listApps:
                let list = shareable.applications.map { app in
                    [
                        "pid": Int(app.processID),
                        "name": app.applicationName,
                    ]
                }
                let json = try JSONSerialization.data(withJSONObject: list, options: [])
                guard let s = String(data: json, encoding: .utf8) else {
                    throw NSError(domain: "lyricwave.sc", code: 90, userInfo: [
                        NSLocalizedDescriptionKey: "failed to encode list json",
                    ])
                }
                print(s)

            case let .capture(args):
                try await runCapture(args: args, shareable: shareable)
            }
        } catch {
            fputs("lyricwave-macos-sc-helper error: \(error)\n", stderr)
            exit(1)
        }
    }
}

private func runCapture(args: CaptureArgs, shareable: SCShareableContent) async throws {
    try FileManager.default.createDirectory(
        at: args.outputURL.deletingLastPathComponent(),
        withIntermediateDirectories: true
    )

    guard let display = shareable.displays.first else {
        throw NSError(domain: "lyricwave.sc", code: 2, userInfo: [
            NSLocalizedDescriptionKey: "no shareable display found for ScreenCaptureKit",
        ])
    }

    let matchedApps = matchApplications(shareable.applications, selectors: args.selectors)
    if matchedApps.isEmpty {
        throw NSError(domain: "lyricwave.sc", code: 3, userInfo: [
            NSLocalizedDescriptionKey: "no running applications matched selectors",
        ])
    }

    let excluded = shareable.applications.filter { app in
        !matchedApps.contains(where: { $0.processID == app.processID })
    }

    let filter = SCContentFilter(display: display, excludingApplications: excluded, exceptingWindows: [])
    let config = SCStreamConfiguration()
    config.capturesAudio = true
    config.excludesCurrentProcessAudio = true
    config.sampleRate = Int(args.sampleRate)
    config.channelCount = Int(args.channels)
    config.width = max(2, Int(display.width))
    config.height = max(2, Int(display.height))

    let tmpM4A = args.outputURL.deletingPathExtension().appendingPathExtension("m4a")
    let output = try AudioCaptureOutput(outputURL: tmpM4A)
    let stream = SCStream(filter: filter, configuration: config, delegate: nil)
    try stream.addStreamOutput(output, type: .audio, sampleHandlerQueue: DispatchQueue(label: "lyricwave.sc.audio"))

    try await stream.startCapture()

    if let seconds = args.seconds {
        try await Task.sleep(nanoseconds: UInt64(seconds) * 1_000_000_000)
    } else {
        throw NSError(domain: "lyricwave.sc", code: 4, userInfo: [
            NSLocalizedDescriptionKey: "seconds is required for macOS process capture in current helper",
        ])
    }

    try await stream.stopCapture()
    try await output.finish()
    try convertM4AToWav(input: tmpM4A, output: args.outputURL)
    try? FileManager.default.removeItem(at: tmpM4A)

    let meta = try readWavMeta(url: args.outputURL)
    let matched = matchedApps.map { app in
        "pid=\(app.processID) name=\(app.applicationName)"
    }

    let result: [String: Any] = [
        "captured_samples": meta.capturedSamples,
        "sample_rate": meta.sampleRate,
        "channels": meta.channels,
        "matched_processes": matched,
    ]

    let jsonData = try JSONSerialization.data(withJSONObject: result, options: [])
    if let jsonStr = String(data: jsonData, encoding: .utf8) {
        print(jsonStr)
    } else {
        throw NSError(domain: "lyricwave.sc", code: 5, userInfo: [
            NSLocalizedDescriptionKey: "failed to encode helper result json",
        ])
    }
}

private func parseArguments() throws -> Mode {
    var listApps = false
    var outPath: String?
    var sampleRate: UInt32 = 48000
    var channels: UInt16 = 2
    var seconds: UInt32?
    var selectors: [Selector] = []

    var idx = 1
    let argv = CommandLine.arguments
    while idx < argv.count {
        let arg = argv[idx]
        switch arg {
        case "--list-apps":
            listApps = true
        case "--out":
            idx += 1
            outPath = valueAt(argv, idx)
        case "--sample-rate":
            idx += 1
            let v = valueAt(argv, idx)
            sampleRate = UInt32(v) ?? sampleRate
        case "--channels":
            idx += 1
            let v = valueAt(argv, idx)
            channels = UInt16(v) ?? channels
        case "--seconds":
            idx += 1
            let v = valueAt(argv, idx)
            seconds = UInt32(v)
        case "--pid":
            idx += 1
            let v = valueAt(argv, idx)
            if let pid = Int32(v) {
                selectors.append(Selector(pid: pid, nameContains: nil))
            }
        case "--name":
            idx += 1
            let v = valueAt(argv, idx)
            selectors.append(Selector(pid: nil, nameContains: v))
        default:
            break
        }
        idx += 1
    }

    if listApps {
        return .listApps
    }

    guard let outPath else {
        throw NSError(domain: "lyricwave.sc", code: 10, userInfo: [
            NSLocalizedDescriptionKey: "missing required --out",
        ])
    }
    if selectors.isEmpty {
        throw NSError(domain: "lyricwave.sc", code: 11, userInfo: [
            NSLocalizedDescriptionKey: "missing selectors (--pid/--name)",
        ])
    }

    return .capture(CaptureArgs(
        outputURL: URL(fileURLWithPath: outPath),
        sampleRate: sampleRate,
        channels: channels,
        seconds: seconds,
        selectors: selectors
    ))
}

private func valueAt(_ argv: [String], _ idx: Int) -> String {
    if idx < argv.count {
        return argv[idx]
    }
    return ""
}

private func matchApplications(_ apps: [SCRunningApplication], selectors: [Selector]) -> [SCRunningApplication] {
    apps.filter { app in
        selectors.contains { selector in
            if let pid = selector.pid, app.processID == pid {
                return true
            }
            if let query = selector.nameContains?.lowercased() {
                return app.applicationName.lowercased().contains(query)
            }
            return false
        }
    }
}

private func convertM4AToWav(input: URL, output: URL) throws {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/afconvert")
    process.arguments = ["-f", "WAVE", "-d", "LEI16", input.path, output.path]
    try process.run()
    process.waitUntilExit()
    if process.terminationStatus != 0 {
        throw NSError(domain: "lyricwave.sc", code: 20, userInfo: [
            NSLocalizedDescriptionKey: "afconvert failed with status \(process.terminationStatus)",
        ])
    }
}

private func readWavMeta(url: URL) throws -> (capturedSamples: Int, sampleRate: Int, channels: Int) {
    let file = try AVAudioFile(forReading: url)
    let fmt = file.processingFormat
    let frames = Int(file.length)
    let channels = Int(fmt.channelCount)
    return (capturedSamples: frames * channels, sampleRate: Int(fmt.sampleRate), channels: channels)
}

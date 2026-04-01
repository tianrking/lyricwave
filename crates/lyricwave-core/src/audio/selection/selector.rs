use crate::audio::InputDeviceInfo;

#[derive(Debug, Clone)]
pub struct SelectedInput {
    pub info: InputDeviceInfo,
    pub reason: String,
}

pub fn select_input_device(
    devices: &[InputDeviceInfo],
    hint: Option<&str>,
    prefer_loopback: bool,
) -> Option<SelectedInput> {
    if devices.is_empty() {
        return None;
    }

    if let Some(hint_text) = hint {
        let query = hint_text.to_lowercase();
        if let Some(device) = devices.iter().find(|d| {
            d.name.to_lowercase().contains(&query) || d.id.to_lowercase().contains(&query)
        }) {
            return Some(SelectedInput {
                info: device.clone(),
                reason: format!("selected by explicit input hint '{hint_text}'"),
            });
        }
    }

    if prefer_loopback
        && let Some(loopback) = devices
            .iter()
            .filter(|d| d.is_loopback_candidate)
            .max_by_key(|d| d.loopback_score)
    {
        return Some(SelectedInput {
            info: loopback.clone(),
            reason: format!(
                "selected best loopback candidate (score={})",
                loopback.loopback_score
            ),
        });
    }

    devices
        .iter()
        .find(|d| d.is_default)
        .cloned()
        .map(|info| SelectedInput {
            info,
            reason: "selected default input device".to_string(),
        })
        .or_else(|| {
            devices.first().cloned().map(|info| SelectedInput {
                info,
                reason: "selected first available input device".to_string(),
            })
        })
}

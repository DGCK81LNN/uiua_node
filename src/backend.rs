use image::{DynamicImage, ImageFormat};
use std::{io::Cursor, mem::take, sync::Mutex};
use uiua::*;

pub enum OutputItem {
    StdOut(String),
    StdErr(String),
    Trace(String),
    Image {
        data: Vec<u8>,
        mime: String,
        label: Option<String>,
    },
    Audio {
        data: Vec<u8>,
        mime: String,
        label: Option<String>,
    },
}

#[derive(Default)]
pub struct MultimediaBackend {
    pub outputs: Mutex<Vec<OutputItem>>,
}

impl MultimediaBackend {
    pub fn new() -> Self {
        Self {
            outputs: vec![].into(),
        }
    }

    pub fn take_outputs(&self) -> Vec<OutputItem> {
        take(&mut self.outputs.lock().unwrap())
    }

    pub fn show_png(&self, png_bytes: Vec<u8>, label: Option<&str>) -> Result<(), String> {
        (self.outputs.lock().unwrap()).push(OutputItem::Image {
            data: png_bytes,
            mime: "image/png".to_string(),
            label: label.map(|s| s.to_string()),
        });
        Ok(())
    }
}

impl SysBackend for MultimediaBackend {
    fn any(&self) -> &dyn std::any::Any {
        self
    }
    fn any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn print_str_stdout(&self, s: &str) -> Result<(), String> {
        let mut outputs = self.outputs.lock().unwrap();
        if let Some(OutputItem::StdOut(prev)) = outputs.last_mut() {
            prev.push_str(s);
        } else {
            outputs.push(OutputItem::StdOut(s.to_string()));
        }
        Ok(())
    }
    fn print_str_stderr(&self, s: &str) -> Result<(), String> {
        let mut outputs = self.outputs.lock().unwrap();
        if let Some(OutputItem::StdErr(prev)) = outputs.last_mut() {
            prev.push_str(s);
        } else {
            outputs.push(OutputItem::StdErr(s.to_string()));
        }
        Ok(())
    }
    fn print_str_trace(&self, s: &str) {
        let mut outputs = self.outputs.lock().unwrap();
        if let Some(OutputItem::Trace(prev)) = outputs.last_mut() {
            prev.push_str(s);
        } else {
            outputs.push(OutputItem::Trace(s.to_string()));
        }
    }
    fn show_image(&self, image: DynamicImage, label: Option<&str>) -> Result<(), String> {
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, ImageFormat::Png)
            .map_err(|e| format!("Failed to show image: {e}"))?;
        self.show_png(bytes.into_inner(), label)
    }
    fn show_gif(&self, gif_bytes: Vec<u8>, label: Option<&str>) -> Result<(), String> {
        (self.outputs.lock().unwrap()).push(OutputItem::Image {
            data: gif_bytes,
            mime: "image/gif".to_string(),
            label: label.map(|s| s.to_string()),
        });
        Ok(())
    }
    fn play_audio(&self, wave_bytes: Vec<u8>, label: Option<&str>) -> Result<(), String> {
        (self.outputs.lock().unwrap()).push(OutputItem::Audio {
            data: wave_bytes,
            mime: "audio/wav".to_string(),
            label: label.map(|s| s.to_string()),
        });
        Ok(())
    }
}

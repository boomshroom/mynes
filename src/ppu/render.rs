use super::Vram;

pub type FrameBuffer = Arc<Mutex<ImageBuffer>>;

macro_rules! yield_ {
	($co:expr) => { $co.yield_(()).await };
}

impl Vram {
	pub (crate) async fn clock<'a>(fb: FrameBuffer, co: Co<'_, (), &mut 'a Vram>) {
		loop {
			let vram = yield_!();
		}
	}
}
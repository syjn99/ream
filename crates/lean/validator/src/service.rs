use tracing::info;

pub struct ValidatorService {}

impl ValidatorService {
    pub async fn new() -> Self {
        ValidatorService {}
    }

    pub async fn start(self) {
        info!("ValidatorService started");

        loop {}
    }
}

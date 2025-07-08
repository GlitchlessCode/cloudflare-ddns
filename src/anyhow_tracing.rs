use std::fmt::Display;

use tracing::{debug, error, info, warn};

pub trait Tracing {
    fn debug(self) -> Self;
    fn debug_success<M>(self, msg: M) -> Self
    where
        M: Display;
    fn info(self) -> Self;
    fn warn_or_error(self, is_error: bool) -> Self;
    fn warn(self) -> Self;
    fn error(self) -> Self;
}

impl<T> Tracing for anyhow::Result<T> {
    #[inline(always)]
    fn debug(self) -> Self {
        self.inspect_err(|err| debug!("{err:?}"))
    }

    #[inline(always)]
    fn debug_success<M>(self, msg: M) -> Self
    where
        M: Display,
    {
        self.inspect(|_| debug!("{msg}"))
    }

    #[inline(always)]
    fn info(self) -> Self {
        self.inspect_err(|err| info!("Encountered an error: {err:?}"))
    }

    #[inline(always)]
    fn warn_or_error(self, is_error: bool) -> Self {
        self.inspect_err(|err| {
            if is_error {
                warn!("{err:?}")
            } else {
                error!("{err:?}")
            }
        })
    }

    #[inline(always)]
    fn warn(self) -> Self {
        self.inspect_err(|err| warn!("{err:?}"))
    }

    #[inline(always)]
    fn error(self) -> Self {
        self.inspect_err(|err| error!("{err:?}"))
    }
}

struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        println!("Cleaning up...");
    }
}

#[cfg(test)]
mod tests {
    use super::CleanupGuard;
    use crate::error::Result;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_defer() -> Result<()> {
        let _guard = CleanupGuard;

        println!("DO some WORK!");
        Ok(())
    }
}

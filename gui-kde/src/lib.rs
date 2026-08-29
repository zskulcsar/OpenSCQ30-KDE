pub mod bridge;

#[cfg(test)]
mod tests {
    #[test]
    fn app_controller_identifies_the_kde_frontend() {
        assert_eq!(super::bridge::frontend_id(), "kde");
    }
}

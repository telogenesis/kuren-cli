fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "proto/kuren/auth/auth.proto",
                "proto/kuren/payment/payment.proto",
                "proto/kuren/wallet/wallet.proto",
                "proto/kuren/social/social.proto",
                "proto/kuren/messaging/messaging.proto",
                "proto/kuren/notifications/notifications.proto",
                "proto/kuren/email/email.proto",
                "proto/kuren/commerce/commerce.proto",
                "proto/kuren/organization/organization.proto",
                "proto/kuren/notes/notes.proto",
            ],
            &["proto", "proto-vendor"],
        )?;
    Ok(())
}

pub mod disk;
pub mod window;

#[cfg(target_os = "android")]
winit::android_activity_main!(android_main);

// Точка входа для Android (тип AndroidApp берем строго из платформенного модуля winit)
#[cfg(target_os = "android")]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    log::info!("Успешный старт нативного приложения через cargo-quad-apk!");

    // Передаем системный контекст экрана во внутренности winit 0.30
    winit::platform::android::activity::set_android_app(app);

    // Запускаем ваш основной UI-код фреймворка
    main_2();
}

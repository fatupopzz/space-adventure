//! En macOS compila el puentecito de la vibracion (`nativo/haptica_mac.m`: GameController y Core
//! Haptics de Apple) y lo enlaza con el juego. En los demas sistemas no hace nada.

fn main() {
    println!("cargo:rerun-if-changed=nativo/haptica_mac.m");
    println!("cargo:rustc-check-cfg=cfg(haptica_apple)");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let salida = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("sin OUT_DIR"));
    let objeto = salida.join("haptica_mac.o");
    let biblioteca = salida.join("libhaptica_mac.a");
    let ok = std::process::Command::new("clang")
        .args(["-fobjc-arc", "-O2", "-mmacosx-version-min=11.0", "-c", "nativo/haptica_mac.m", "-o"])
        .arg(&objeto)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        // Sin clang (o si falla), el juego compila igual y vibra con SDL si puede.
        println!("cargo:warning=no se pudo compilar nativo/haptica_mac.m: sin hapticos de Apple");
        return;
    }
    let _ = std::fs::remove_file(&biblioteca);
    let ok = std::process::Command::new("ar").arg("crs").arg(&biblioteca).arg(&objeto).status().map(|s| s.success()).unwrap_or(false);
    if !ok {
        println!("cargo:warning=no se pudo armar libhaptica_mac.a");
        return;
    }
    println!("cargo:rustc-link-search=native={}", salida.display());
    println!("cargo:rustc-link-lib=static=haptica_mac");
    for framework in ["Foundation", "GameController", "CoreHaptics"] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
    println!("cargo:rustc-cfg=haptica_apple");
}

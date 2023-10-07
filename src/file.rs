use std::ffi::OsString;
use std::{
    env, fs,
    string::{FromUtf8Error, String},
};
use std::{io, str};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ReadFileError {
    LACK_PARAM_ERROR,
    FILE_NAME_ERROR(OsString),
    FILE_NOT_FIND(io::Error),
    File_Encode_Error(FromUtf8Error),
}

pub fn read_file_content(path: &str) -> Result<String, ReadFileError> {
    let uf = fs::read(path).map_err(|e| ReadFileError::FILE_NOT_FIND(e))?;
    String::from_utf8(uf).map_err(|e| ReadFileError::File_Encode_Error(e))
}

pub fn get_file_path() -> Result<String, ReadFileError> {
    env::args_os()
        .nth(1)
        .ok_or(ReadFileError::LACK_PARAM_ERROR)?
        .into_string()
        .map_err(|e| ReadFileError::FILE_NAME_ERROR(e))
}

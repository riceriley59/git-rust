use crate::cli::Kind;

use std::fs;
use std::ffi::CStr;
use std::io::{self, prelude::*, BufReader};

use anyhow::{Context, Result};
use sha1::{Sha1, Digest};

use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub fn init() -> Result<()> {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory");

    Ok(())
}

pub fn hash_object(filename: String, write: bool) -> Result<()> {
    anyhow::ensure!(
        write,
        "Need to write with the 'w' command"
    );

    let mut object_file = fs::File::open(filename)?;
    let mut buf = Vec::new();
    object_file.read_to_end(&mut buf)?;

    let header = format!("blob {}\0", buf.len());
    let mut full_data = Vec::new();
    full_data.extend_from_slice(header.as_bytes());
    full_data.extend_from_slice(&buf);

    let mut hasher = Sha1::new();
    hasher.update(&buf);
    let result = hasher.finalize();

    let hash_hex = format!("{:x}", result);
    println!("{}", hash_hex);

    let dir = format!(".git/objects/{}", &hash_hex[..2]);
    let file_name = &hash_hex[2..];
    let path = format!("{}/{}", dir, file_name);
    fs::create_dir_all(&dir)?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&full_data)?;
    let compressed_data = encoder.finish()?;

    let mut output_file = fs::File::create(&path)?;
    output_file.write_all(&compressed_data)?;

    Ok(())
}

pub fn ls_tree(tree_hash: String, name_only: bool) -> Result<()> {
    let tree_file = fs::File::open(format!(
        ".git/objects/{}/{}",
        &tree_hash[..2],
        &tree_hash[2..],
    ))
    .with_context(|| format!("Failed to open tree object file for hash {tree_hash}"))?;

    let zdec = ZlibDecoder::new(BufReader::new(tree_file));
    let mut z_buf_reader = BufReader::new(zdec);
    let mut buf = Vec::new();

    z_buf_reader
        .read_until(0, &mut buf)
        .context("Failed to read header .git/objects")?;

    let header = CStr::from_bytes_with_nul(&buf)?
        .to_str()
        .context(".git/objects file header isn't valid UTF-8")?;

    let (kind, size) = header
        .split_once(' ')
        .context(".git/objects file headers does not start with a know type")?;

    let kind = match kind {
        "tree" => Kind::Tree,
        _ => anyhow::bail!("Doesn't support printing '{kind}' yet"),
    };

    let size = size
        .parse::<usize>()
        .with_context(|| format!(".git/objects file header has invalid size: {size}"))?;

    buf.clear();
    buf.resize(size, 0);

    z_buf_reader
        .read_exact(&mut buf)
        .context("Failed to read the actual contents of .git/objects file")?;

    let mut trailing = [0; 1];
    let trailing_bytes = z_buf_reader
        .read(&mut trailing)
        .context("Failed to validate EOF in file")?;
    anyhow::ensure!(
        trailing_bytes == 0,
        ".git/object had {trailing_bytes} trailing bytes"
    );

    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    match kind {
        Kind::Tree => stdout_lock
            .write_all(&buf)
            .context("Failed to write contents to stdout")?,
        _ => anyhow::bail!("Doesn't support printing"),
    }

    Ok(())
}

pub fn cat_file(object_hash: String, pretty_print: bool) -> Result<()> {
    anyhow::ensure!(
        pretty_print,
        "Need to set pretty print on for this command with 'p'"
    );

    let object_file = fs::File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..]
    ))
    .with_context(|| format!("Failed to open object file for hash {object_hash}"))?;

    let zdec = ZlibDecoder::new(BufReader::new(object_file));
    let mut z_buf_reader = BufReader::new(zdec);
    let mut buf = Vec::new();

    z_buf_reader
        .read_until(0, &mut buf)
        .context("Failed to read header from .git/objects")?;

    let header = CStr::from_bytes_with_nul(&buf)?
        .to_str()
        .context(".git/objects file header isn't valid UTF-8")?;

    let (kind, size) = header
        .split_once(' ')
        .context(".git/objects file header does not start with a known type")?;

    let kind = match kind {
        "blob" => Kind::Blob,
        _ => anyhow::bail!("Doesn't support printing '{kind}' yet"),
    };

    let size = size
        .parse::<usize>()
        .with_context(|| format!(".git/objects file header has invalid size: {size}"))?;

    buf.clear();
    buf.resize(size, 0);

    z_buf_reader
        .read_exact(&mut buf)
        .context("Failed to read the actual contents of .git/objects file")?;

    let mut trailing = [0; 1];
    let trailing_bytes = z_buf_reader
        .read(&mut trailing)
        .context("Failed to validate EOF in file")?;
    anyhow::ensure!(
        trailing_bytes == 0,
        ".git/object had {trailing_bytes} trailing bytes"
    );

    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    match kind {
        Kind::Blob => stdout_lock
            .write_all(&buf)
            .context("Failed to write contents to stdout")?,
        _ => anyhow::bail!("Doesn't support printing"),
    }

    Ok(())
}

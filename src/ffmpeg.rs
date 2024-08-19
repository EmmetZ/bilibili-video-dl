use std::{
    fs,
    path::{Path, PathBuf},
};

use ffmpeg_next::{codec, encoder, format, media};

use crate::http::Result;

/// merge audio and video
/// ### Parameters
/// - a_path: the path to audio
/// - v_path: the path to video
/// - o_path: the path to write output video
pub fn merge(a_path: &Path, v_path: &Path, o_path: &PathBuf) -> Result<()> {
    if o_path.exists() {
        fs::remove_file(o_path)?;
    }
    fs::File::create(o_path)?;

    let mut ia_ctx = format::input(&a_path)?;
    let mut iv_ctx = format::input(&v_path)?;
    let mut octx = format::output(o_path)?;

    let iv_stream = iv_ctx.streams().best(media::Type::Video).unwrap();
    let ia_stream = ia_ctx.streams().best(media::Type::Audio).unwrap();

    let stream_index = [iv_stream.index(), ia_stream.index()];
    let stream_time_base = [iv_stream.time_base(), ia_stream.time_base()];

    for stream in [iv_stream, ia_stream].iter() {
        let mut o_stream = octx.add_stream(encoder::find(codec::Id::None)).unwrap();
        o_stream.set_parameters(stream.parameters());
        unsafe {
            (*o_stream.parameters().as_mut_ptr()).codec_tag = 0;
        }
    }

    octx.set_metadata(iv_ctx.metadata().to_owned());
    octx.write_header().unwrap();

    for (i, packer_iter) in [iv_ctx.packets(), ia_ctx.packets()].into_iter().enumerate() {
        for (stream, mut packet) in packer_iter {
            if stream.index() == stream_index[i] {
                let ost = octx.stream(i).unwrap();
                packet.rescale_ts(stream_time_base[i], ost.time_base());
                packet.set_position(-1);
                packet.set_stream(i);
                packet.write_interleaved(&mut octx).unwrap();
            }
        }
    }
    octx.write_trailer().unwrap();
    Ok(())
}

use clap::Parser;
use ffmpeg_next as ffpmeg;
use ffmpeg_next::{frame::Video, media::Type};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: String,
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let mut input = ffpmeg::format::input(&args.path).expect("open file");

    let video_stream_ids: Vec<_> = input
        .streams()
        .filter(|stream| stream.parameters().medium() == Type::Video)
        .map(|stream| stream.index())
        .collect();

    'streams: for stream_id in video_stream_ids {
        let stream = input.stream(stream_id).unwrap();
        let context_decoder = ffpmeg::codec::context::Context::from_parameters(stream.parameters())
            .expect("context from parametersx");

        let mut decoder = context_decoder
            .decoder()
            .video()
            .expect("get decoder instance");

        let mut frame_counter = 0;
        let mut kf1 = None;
        let mut kf2 = None;
        let mut gop_size = None;

        for (stream, packet) in input.packets() {
            if stream.index() == stream_id {
                decoder.send_packet(&packet).expect("send_packet");
                let mut frame = Video::empty();

                while decoder.receive_frame(&mut frame).is_ok() {
                    if frame.is_key() {
                        if kf1.is_none() {
                            kf1 = Some(frame_counter);
                        } else if kf2.is_none() {
                            kf2 = Some(frame_counter);
                        } else {
                            kf1 = kf2;
                            kf2 = Some(frame_counter);
                        }

                        if let (Some(ki1), Some(ki2)) = (kf1, kf2) {
                            if let Some(cur_gop_size) = gop_size {
                                if cur_gop_size != (ki2 - ki1) {
                                    println!(
                                        "Invalid keyframe GOP size {} expected {} at frame index {} in stream {}",
                                        ki2 - ki1,
                                        cur_gop_size,
                                        frame_counter,
                                        stream_id
                                    );

                                    continue 'streams;
                                } else if args.verbose {
                                    println!(
                                        "Frame {} is a keyframe [diff: {}]",
                                        frame_counter,
                                        ki2 - ki1
                                    );
                                }
                            } else {
                                gop_size = Some(ki2 - ki1);
                            }
                        }
                    }
                    frame_counter += 1;
                }
            }
        }
        if let Some(gop_size) = gop_size {
            println!(
                "Stream {} has a consistent GOP size of {}",
                stream_id, gop_size
            );
        }
    }
}

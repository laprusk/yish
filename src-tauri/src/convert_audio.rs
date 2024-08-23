use tauri::Manager;
use vvcapi::VoicevoxCore;
use yomiage;
use hound::{WavReader, WavWriter};
use chrono::Local;
use std::io::Cursor;

pub fn convert_audio(
    core: &VoicevoxCore,
    yp: yomiage::Problem,
    speed_scale: f64,
    output_path: &str,
    app_handle: &tauri::AppHandle,
    count_problems: u32,
) -> Result<(), String> {
    let speaker_id = 13;

    if !core.is_model_loaded(speaker_id) {
        println!("Loading model...");
        app_handle.emit_all(
            "progress",
            "モデルをロード中...",
        ).unwrap();
        core.load_model(13).unwrap();
        println!("Model loaded");
        app_handle.emit_all(
            "progress",
            format!("生成中... 0/{}", count_problems),
        ).unwrap();
    }

    let mut path = std::path::PathBuf::from(output_path);
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }

    path.push(get_file_name(yp.config));
    let output_path = path.to_str().unwrap();

    // Meta
    let mut aq_meta = core.audio_query(&yp.script_meta(), speaker_id).unwrap();
    aq_meta.pitch_scale += 0.1;
    let wav_meta = core.synthesis(aq_meta, speaker_id).unwrap();

    // Problem
    let mut aq_problem = core.audio_query(&yp.script_problem(), speaker_id).unwrap();
    aq_problem.speed_scale = speed_scale;
    aq_problem.pitch_scale += 0.1;
    let wav_problem = core.synthesis(aq_problem, speaker_id).unwrap();

    // Answer
    let mut aq_answer = core.audio_query(&yp.script_answer(), speaker_id).unwrap();
    aq_answer.pitch_scale += 0.1;
    let wav_answer = core.synthesis(aq_answer, speaker_id).unwrap();

    // Concat
    let reader_meta = WavReader::new(Cursor::new(wav_meta)).unwrap();
    let reader_problem = WavReader::new(Cursor::new(wav_problem)).unwrap();
    let reader_answer = WavReader::new(Cursor::new(wav_answer)).unwrap();

    let spec = reader_meta.spec();
    let mut writer = WavWriter::create(output_path, spec).unwrap();

    // 0.5s silence and 3s silence
    let sec1: f64 = 0.5;
    let sec2: f64 = 3.0;
    let silence_duration = spec.sample_rate;
    let silence1 = vec![0_i16; (silence_duration as f64 * sec1) as usize];
    let silence2 = vec![0_i16; (silence_duration as f64 * sec2) as usize];

    for sample in reader_meta.into_samples::<i16>() {
        writer.write_sample(sample.unwrap()).unwrap();
    }
    for &sample in &silence1 {
        writer.write_sample(sample).unwrap();
    }
    for sample in reader_problem.into_samples::<i16>() {
        writer.write_sample(sample.unwrap()).unwrap();
    }
    for &sample in &silence2 {
        writer.write_sample(sample).unwrap();
    }
    for sample in reader_answer.into_samples::<i16>() {
        writer.write_sample(sample.unwrap()).unwrap();
    }

    writer.finalize().unwrap();

    Ok(())
}

fn get_file_name(yomiage_config: yomiage::Config) -> String {
    let min_digit = yomiage_config.min_digit.to_string();
    let max_digit = yomiage_config.max_digit.to_string();
    let length = yomiage_config.length.to_string();
    let problem_type = if yomiage_config.subtractions == 0 {
        String::from("加算")
    } else {
        String::from("加減算")
    };
    let timestamp = get_timestamp();

    format!("{}-{}-{}-{}-{}.wav", min_digit, max_digit, length, timestamp, problem_type)
}

fn get_timestamp() -> String {
    Local::now().format("%Y%m%d%H%M%S").to_string()
}
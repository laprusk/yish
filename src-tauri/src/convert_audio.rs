use tauri::Manager;
use vvcapi::VoicevoxCore;
use yomiage;
use hound::{WavReader, WavWriter};
use std::io::Cursor;
use crate::commands::GenConfig;

pub fn convert_audio(
    app_handle: &tauri::AppHandle,
    yp: yomiage::Problem,
    GenConfig { count_problems, speed_scale, output_path, .. }: GenConfig,
    dir_name: &str,
    file_name: &str,
) -> Result<(), String> {
    let speaker_id = 13;
    let core = app_handle.state::<VoicevoxCore>();

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
    path.push(dir_name);
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }

    path.push(file_name);
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

    // make silence
    let make_silence = |sec: f64| -> Vec<i16> {
        vec![0_i16; (spec.sample_rate as f64 * sec) as usize]
    };
    let sec1: f64 = 0.5;
    let sec2: f64 = 3.0;
    let sec3: f64 = 1.0;
    let silence1 = make_silence(sec1);
    let silence2 = make_silence(sec2);
    let silence3 = make_silence(sec3);

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
    for &sample in &silence3 {
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();

    Ok(())
}

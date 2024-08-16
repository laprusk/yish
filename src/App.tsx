import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  type YomiageConfig = {
    min_digit: number;
    max_digit: number;
    length: number;
    subtractions: number;
    allow_negative: boolean;
  };

  type GenConfig = {
    count_problems: number;
    gen_type: number;
    speed_scale: number;
    output_path: string;
  };

  // useState
  const [yomiageConfig, setYomiageConfig] = useState<YomiageConfig>({
    min_digit: 3,
    max_digit: 6,
    length: 10,
    subtractions: 3,
    allow_negative: false,
  });
  const [genConfig, setGenConfig] = useState<GenConfig>({
    count_problems: 4,
    gen_type: 2,
    speed_scale: 1.0,
    output_path: "",
  });
  const [progress, setProgress] = useState<string>("");
  const [isGenerating, setIsGenerating] = useState<boolean>(false);

  // useEffect
  useEffect(() => {
    async function fetchDefaultPath() {
      const path = await getDefaultPath();
      setGenConfig((prevConfig: GenConfig) => ({
        ...prevConfig,
        output_path: path,
      }));
    }
    fetchDefaultPath();
  }, []);

  const handleYomiageConfigChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setYomiageConfig(prevConfig => ({
      ...prevConfig,
      [name]: name === "allow_negative" ? e.target.checked : parseInt(value),
    }));
  }

  const handleGenConfigChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setGenConfig(prevConfig => ({
      ...prevConfig,
      [name]: name === "output_path" ? value
        : name === "speed_scale" ? parseFloat(value)
        : parseInt(value),
    }));
    console.log(genConfig);
  }

  async function getDefaultPath(): Promise<string> {
    const path: string = await invoke("get_default_path");
    return path;
  }

  async function generateAudio(yomiageConfig: YomiageConfig, genConfig: GenConfig) {
    console.log("generateAudio");

    if (isGenerating) {
      return;
    }
    setIsGenerating(true);

    console.log(yomiageConfig);
    console.log(genConfig);

    const unlisten_prog = await listen<string>("progress", (event) => {
      setProgress(event.payload);
    });

    const unlisten_finish = await listen("finish", (_) => {
      setIsGenerating(false);
      unlisten_prog();
      unlisten_finish();
    });

    const result = await invoke("generate_audio", {
      yomiageConfig,
      genConfig,
    });
    console.log(result);
  }

  return (
    <div className="container">
      <form
        onSubmit={(e) => {
          e.preventDefault();
          getDefaultPath();
          generateAudio(yomiageConfig, genConfig);
        }}
      >
        <div className="row">
          <label>桁数</label>
          <input
            type="number"
            name="min_digit"
            value={yomiageConfig.min_digit}
            onChange={handleYomiageConfigChange}
          />
          <span className="wave">〜</span>
          <input
            type="number"
            name="max_digit"
            value={yomiageConfig.max_digit}
            onChange={handleYomiageConfigChange}
          />
        </div>
        <div className="row">
          <label>口数</label>
          <input
            type="number"
            name="length"
            value={yomiageConfig.length}
            onChange={handleYomiageConfigChange}
          />
        </div>
        <div className="row">
          <label>引き算回数</label>
          <input
            type="number"
            name="subtractions"
            value={yomiageConfig.subtractions}
            onChange={handleYomiageConfigChange}
          />
        </div>
        <div className="row">
          <input
            type="checkbox"
            name="allow_negative"
            className="checkbox"
            checked={yomiageConfig.allow_negative}
            onChange={handleYomiageConfigChange}
          />
          <span>マイナス算を許可</span>
        </div>
        <div className="separator"></div>
        <div className="row">
          <label>問題数</label>
          <input
            type="number"
            name="count_problems"
            value={genConfig.count_problems}
            onChange={handleGenConfigChange}
          />
        </div>
        <div className="row">
          <label>生成タイプ</label>
          <select
            name="gen_type"
            value={genConfig.gen_type}
            onChange={handleGenConfigChange}
          >
            <option value="0">加算のみ</option>
            <option value="1">加減算のみ</option>
            <option value="2">加算と加減算を交互に生成</option>
          </select>
        </div>
        <div className="row">
          <label>速度</label>
          <input
            type="number"
            name="speed_scale"
            value={genConfig.speed_scale}
            onChange={handleGenConfigChange}
          />
        </div>
        <div className="row">
          <label>出力先</label>
          <input
            type="text"
            name="output_path"
            className="path"
            value={genConfig.output_path}
            onChange={handleGenConfigChange}
          />
        </div>
        <button type="submit">
          {isGenerating ? "生成中..." : "生成"}
        </button>
      </form>
      <p>{progress}</p>
    </div>
  );
}

export default App;

import { useState, useEffect } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Play, FolderOpen, RefreshCw, CheckCircle2, Circle, Plus, Minus, Shuffle } from "lucide-react";

type MediaType = "image" | "video";

interface MediaItem {
  path: string;
  media_type: MediaType;
  included: boolean;
  thumbnail?: string; 
  displayUrl?: string; // URL for <img> src
}

interface Config {
  wallpaper_dir: string | null;
  interval_seconds: number;
  media: MediaItem[];
  random: boolean;
}

export default function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [visibleCount, setVisibleCount] = useState(30);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const cfg = await invoke<Config>("get_config");
      
      // Process media items
      const mediaWithUrls = await Promise.all(
        cfg.media.map(async (item) => {
          let thumbnail = undefined;
          if (item.media_type === "video") {
            try {
              thumbnail = await invoke<string>("generate_thumbnail", { videoPath: item.path });
            } catch (e) {
              console.error("Failed to generate thumb for", item.path, e);
            }
          }
          
          // Use convertFileSrc for efficient local file access
          // If video and thumbnail failed, we don't have a displayUrl
          let displayUrl = undefined;
          if (item.media_type === "image") {
            displayUrl = convertFileSrc(item.path);
          } else if (thumbnail) {
            displayUrl = convertFileSrc(thumbnail);
          }

          return { ...item, thumbnail, displayUrl };
        })
      );
      
      setConfig({ ...cfg, media: mediaWithUrls });
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async (newConfig: Config) => {
    try {
      await invoke("save_config", { config: newConfig });
      setConfig(newConfig);
    } catch (e) {
      console.error(e);
    }
  };

  const selectDirectory = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    
    if (selected && config) {
      const newConfig = { ...config, wallpaper_dir: selected as string };
      await saveConfig(newConfig);
      // We might need to wait for daemon to sync, or we can reload after a delay
      setTimeout(loadConfig, 1000);
    }
  };

  const toggleInclude = async (path: string) => {
    if (!config) return;
    const newMedia = config.media.map(m => 
      m.path === path ? { ...m, included: !m.included } : m
    );
    const newConfig = { ...config, media: newMedia };
    await saveConfig(newConfig);
  };

  const toggleRandom = async () => {
    if (!config) return;
    const newConfig = { ...config, random: !config.random };
    await saveConfig(newConfig);
  };

  const playNext = async () => {
    try {
      await invoke("next_wallpaper");
    } catch (e) {
      console.error(e);
    }
  };

  const updateInterval = async (val: string) => {
    if (!config) return;
    const secs = parseInt(val, 10);
    if (!isNaN(secs)) {
      const newConfig = { ...config, interval_seconds: secs };
      await saveConfig(newConfig);
    }
  };

  if (loading || !config) {
    return <div className="flex h-screen items-center justify-center text-zinc-400">Loading...</div>;
  }

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-zinc-950 text-zinc-100 p-6 font-sans">
      <header className="flex justify-between items-center mb-8 border-b border-zinc-800 pb-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white mb-1">Smart Wallpaper</h1>
          <p className="text-sm text-zinc-400">
            {config.wallpaper_dir ? `Watching: ${config.wallpaper_dir}` : "No directory selected"}
          </p>
        </div>
        
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 bg-zinc-900/30 border border-zinc-800/50 rounded-xl px-4 py-2 backdrop-blur-sm shadow-sm">
            <div className="flex items-center gap-2">
              <RefreshCw size={14} className="text-indigo-400 animate-pulse" />
              <span className="text-sm font-medium text-zinc-300">Interval</span>
            </div>
            
            <div className="flex items-center bg-zinc-950/50 border border-zinc-800 rounded-lg p-0.5 shadow-inner mx-1">
              <button 
                onClick={() => updateInterval((config.interval_seconds - 1).toString())}
                className="p-1.5 hover:bg-zinc-800 text-zinc-400 hover:text-white rounded-md transition-all active:scale-90"
                title="Decrease interval"
              >
                <Minus size={14} />
              </button>
              
              <input 
                type="number"
                value={config.interval_seconds}
                onChange={(e) => updateInterval(e.target.value)}
                className="bg-transparent text-sm font-bold text-white w-10 text-center outline-none [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                min="1"
              />
              
              <button 
                onClick={() => updateInterval((config.interval_seconds + 1).toString())}
                className="p-1.5 hover:bg-zinc-800 text-zinc-400 hover:text-white rounded-md transition-all active:scale-90"
                title="Increase interval"
              >
                <Plus size={14} />
              </button>
            </div>
            
            <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">Secs</span>
          </div>

          <button 
            onClick={toggleRandom}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all shadow-sm font-medium text-sm border active:scale-95 ${
              config.random 
                ? "bg-indigo-600/20 text-indigo-400 border-indigo-500/30 hover:bg-indigo-600/30" 
                : "bg-zinc-800 text-zinc-400 border-transparent hover:bg-zinc-700"
            }`}
            title={config.random ? "Random order enabled" : "Sequential order enabled"}
          >
            <Shuffle size={16} className={config.random ? "text-indigo-400" : "text-zinc-500"} />
            {config.random ? "Random" : "Sequential"}
          </button>

          <button 
            onClick={loadConfig}
            className="flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-all shadow-sm font-medium text-sm border border-transparent active:scale-95"
          >
            <RefreshCw size={16} />
            Refresh
          </button>

          <button 
            onClick={selectDirectory}
            className="flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-all shadow-sm font-medium text-sm border border-transparent active:scale-95"
          >
            <FolderOpen size={16} />
            Select Folder
          </button>
          
          <button 
            onClick={playNext}
            className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg transition-all shadow-md font-medium text-sm border border-indigo-500/50 active:scale-95"
          >
            <Play size={16} className="fill-current" />
            Play Next Now
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-y-auto min-h-0 pr-2 custom-scrollbar pb-8">
        {config.media.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-zinc-500 gap-4">
            <RefreshCw size={48} className="opacity-20" />
            <p>No media found. Select a folder to begin.</p>
          </div>
        ) : (
          <div className="flex flex-col gap-6">
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
              {config.media.slice(0, visibleCount).map((item) => {
                return (
                  <div 
                    key={item.path} 
                    className={`group relative aspect-video rounded-xl overflow-hidden cursor-pointer border-2 transition-all ${
                      item.included ? "border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]" : "border-transparent bg-zinc-900 opacity-60 hover:opacity-100"
                    }`}
                    onClick={() => toggleInclude(item.path)}
                  >
                    <img 
                      src={item.displayUrl || "https://placehold.co/600x400/18181b/52525b?text=No+Preview"} 
                      className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                      alt="wallpaper thumbnail" 
                      loading="lazy"
                    />
                    
                    <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity flex flex-col justify-end p-3">
                      <p className="text-xs truncate text-white/90 drop-shadow-md font-medium">
                        {item.path.split("/").pop()}
                      </p>
                    </div>

                    <div className="absolute top-2 right-2 drop-shadow-lg">
                      {item.included ? (
                        <CheckCircle2 className="text-indigo-400 fill-indigo-400/20" size={24} />
                      ) : (
                        <Circle className="text-white/40 group-hover:text-white/60" size={24} />
                      )}
                    </div>
                    
                    {item.media_type === "video" && (
                      <div className="absolute top-2 left-2 bg-black/60 backdrop-blur-md px-2 py-0.5 rounded text-[10px] font-bold text-white border border-white/10 uppercase tracking-wider shadow-sm">
                        Video
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
            
            {visibleCount < config.media.length && (
              <div className="flex justify-center mt-4">
                <button 
                  onClick={() => setVisibleCount(v => v + 30)}
                  className="px-6 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-all shadow-sm font-medium text-sm border border-transparent active:scale-95 flex items-center gap-2"
                >
                  <FolderOpen size={16} />
                  Load More Wallpapers ({config.media.length - visibleCount} remaining)
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

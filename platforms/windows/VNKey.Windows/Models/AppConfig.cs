using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;

namespace VNKey.Windows.Models
{
    public class AppConfig
    {
        public InputMode CurrentInputMode { get; set; } = InputMode.Telex;
        public bool IsVietnameseMode { get; set; } = true;
        public bool? IsDarkMode { get; set; } = null;
        
        // Settings tab
        public bool ModernTone { get; set; } = true;
        public bool FreeMarking { get; set; } = true;
        public bool SpellCheck { get; set; } = false;
        public bool AutoRestore { get; set; } = true;
        public bool AllowForeignConsonants { get; set; } = true;
        public int SimulationDelay { get; set; } = 0;

        // System tab
        public bool StartWithWindows { get; set; } = false;
        public bool ShorthandWhileOff { get; set; } = false;
        public bool ShorthandAutoCase { get; set; } = true;
        public bool HideWhenStarted { get; set; } = false;
        public bool ShowTrayNotify { get; set; } = true;
        public int NavLayout { get; set; } = 0; // 0: Vertical, 1: Horizontal
        public bool BeepOnModeChange { get; set; } = false;
        public bool IsDevModeEnabled { get; set; } = false;

        // Auto capitalization
        public bool AutoCapitalizeSentence { get; set; } = false;
        public bool AutoCapitalizeEnter { get; set; } = false;
        public bool SmartLiteralMode { get; set; } = true;
        public string SwitchShortcut { get; set; } = "Alt+Z";
        public string CustomShortcut { get; set; } = "Ctrl+Alt+S";

        // Shorthand
        public List<ShorthandItem> ShorthandEntries { get; set; } = new List<ShorthandItem>();

        private static string ConfigPath => Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "config.json");

        public static AppConfig Load()
        {
            try
            {
                if (File.Exists(ConfigPath))
                {
                    string json = File.ReadAllText(ConfigPath);
                    return JsonSerializer.Deserialize<AppConfig>(json) ?? new AppConfig();
                }
            }
            catch { }
            return new AppConfig();
        }

        public void Save()
        {
            try
            {
                string json = JsonSerializer.Serialize(this, new JsonSerializerOptions { WriteIndented = true });
                File.WriteAllText(ConfigPath, json);
            }
            catch { }
        }
    }

    public class ShorthandItem
    {
        public string Macro { get; set; } = "";
        public string Expansion { get; set; } = "";
    }
}

using System;
using System.Collections.ObjectModel;
using System.Text.Json;
using System.Windows.Threading;
using VNKey.Windows.Infrastructure;
using VNKey.Windows.Services;

namespace VNKey.Windows.ViewModels
{
    public class DiagnosticsViewModel : ViewModelBase
    {
        private readonly IEngineService _engineService;
        private readonly DispatcherTimer _timer;

        private string _activeWindow = "N/A";
        private string _keyboardLayout = "N/A";
        private string _phonology = "[] + [] + [] (Tone: 0)";
        private string _latency = "0.00ms";
        private string _bufferText = "Buffer: | Reconstructed:";
        private string _reconstructedText = "-";
        private string _stateText = "Normal";
        private int _score = 0;
        private string _modesText = "Telex | V-Mode";
        private bool _isShorthandActive;
        private bool _isDictionaryActive;
        private bool _isPopOut;
        private string _lastRawKey = "None";
        private string _lastAction = "Initial";

        public string ActiveWindow { get => _activeWindow; set => SetProperty(ref _activeWindow, value); }
        public string KeyboardLayout { get => _keyboardLayout; set => SetProperty(ref _keyboardLayout, value); }
        public string Phonology { get => _phonology; set => SetProperty(ref _phonology, value); }
        public string Latency { get => _latency; set => SetProperty(ref _latency, value); }
        public string BufferText { get => _bufferText; set => SetProperty(ref _bufferText, value); }
        public string ReconstructedText { get => _reconstructedText; set => SetProperty(ref _reconstructedText, value); }
        public string StateText { get => _stateText; set => SetProperty(ref _stateText, value); }
        public int Score { get => _score; set => SetProperty(ref _score, value); }
        public string ModesText { get => _modesText; set => SetProperty(ref _modesText, value); }
        public bool IsShorthandActive { get => _isShorthandActive; set => SetProperty(ref _isShorthandActive, value); }
        public bool IsDictionaryActive { get => _isDictionaryActive; set => SetProperty(ref _isDictionaryActive, value); }
        public bool IsPopOut { get => _isPopOut; set => SetProperty(ref _isPopOut, value); }
        public string LastRawKey { get => _lastRawKey; set => SetProperty(ref _lastRawKey, value); }
        public string LastAction { get => _lastAction; set => SetProperty(ref _lastAction, value); }

        public ObservableCollection<string> DecisionLogs { get; } = new ObservableCollection<string>();
        public ObservableCollection<string> EventLogs { get; } = new ObservableCollection<string>();

        public event Action? DevModeToggleRequested;

        public DiagnosticsViewModel(IEngineService engineService)
        {
            _engineService = engineService;
            
            LogEvent("VNKey Diagnostics Initialized.");
            LogEvent($".NET Version: {Environment.Version}");
            LogEvent($"OS Theme: {(App.MainViewModel?.IsDarkMode == true ? "Dark" : "Light")}");

            // Subscribe to engine errors
            _engineService.OnError += (msg) => LogEvent(msg);

            _timer = new DispatcherTimer();
            _timer.Interval = TimeSpan.FromMilliseconds(100);
            _timer.Tick += (s, e) => UpdateDiagnostics();
            _timer.Start();
        }

        private void UpdateDiagnostics()
        {
            string json = _engineService.GetDiagnosticInfo();
            if (string.IsNullOrEmpty(json)) return;

            try
            {
                using (var doc = JsonDocument.Parse(json))
                {
                    var root = doc.RootElement;
                    
                    ActiveWindow = root.GetProperty("active_window").GetString() ?? "N/A";
                    KeyboardLayout = root.GetProperty("keyboard_layout").GetString() ?? "N/A";
                    Latency = root.GetProperty("processing_time_ms").GetDouble().ToString("F2") + "ms";
                    
                    // Phonology — Rust fields: onset, vowel, coda, tone
                    string onset = root.GetProperty("onset").GetString() ?? "";
                    string vowel = root.GetProperty("vowel").GetString() ?? "";
                    string coda = root.GetProperty("coda").GetString() ?? "";
                    int tone = root.GetProperty("tone").GetInt32();
                    Phonology = $"[{onset}] + [{vowel}] + [{coda}] (Tone: {tone})";

                    // Buffer — Rust fields: buffer, reconstructed
                    string buffer = root.GetProperty("buffer").GetString() ?? "";
                    string reconstructed = root.GetProperty("reconstructed").GetString() ?? "";
                    BufferText = $"Buffer: {buffer} | Reconstructed: {reconstructed}";
                    ReconstructedText = reconstructed;
                    
                    // State & Score — Rust fields: literal_mode, mode, phonetic_score
                    bool literalMode = root.GetProperty("literal_mode").GetBoolean();
                    string mode = root.GetProperty("mode").GetString() ?? "Telex";
                    StateText = literalMode ? "Literal (English)" : mode;
                    Score = root.GetProperty("phonetic_score").GetInt32();

                    // Shorthand & Dictionary
                    IsShorthandActive = root.TryGetProperty("is_shorthand", out var ish) && ish.GetBoolean();
                    IsDictionaryActive = root.TryGetProperty("in_dictionary", out var idi) && idi.GetBoolean();

                    // Modes Context
                    string vMode = root.TryGetProperty("vietnamese_mode", out var vm) && vm.GetBoolean() ? "V-Mode" : "E-Mode";
                    ModesText = $"{mode} | {vMode}";

                    // Raw Tracking
                    LastRawKey = root.TryGetProperty("last_raw_key", out var pk) ? pk.GetString() ?? "None" : "None";
                    LastAction = root.TryGetProperty("last_action", out var pa) ? pa.GetString() ?? "Initial" : "Initial";
                    
                    // Decision Logs
                    DecisionLogs.Clear();
                    foreach (var log in root.GetProperty("decision_log").EnumerateArray())
                    {
                        DecisionLogs.Add(log.GetString() ?? "");
                    }

                    // Dev mode activation — Rust detects "vnkdev" in raw keystroke history
                    if (root.GetProperty("is_dev_password_matched").GetBoolean())
                    {
                        DevModeToggleRequested?.Invoke();
                    }
                }
            }
            catch (Exception ex)
            {
                // Show error directly on visible fields
                BufferText = $"ERROR: {ex.GetType().Name}";
                ReconstructedText = ex.Message;
                App.Current?.Dispatcher?.Invoke(() =>
                {
                    if (EventLogs.Count == 0 || !EventLogs[0].Contains(ex.GetType().Name))
                    {
                        EventLogs.Insert(0, $"[{DateTime.Now:HH:mm:ss}] ERROR: {ex.GetType().Name}: {ex.Message}");
                        if (EventLogs.Count > 100) EventLogs.RemoveAt(100);
                    }
                });
            }
        }

        public void LogEvent(string message)
        {
            App.Current.Dispatcher.Invoke(() => 
            {
                EventLogs.Insert(0, $"[{DateTime.Now:HH:mm:ss}] {message}");
                if (EventLogs.Count > 100) EventLogs.RemoveAt(100);
            });
        }
    }
}

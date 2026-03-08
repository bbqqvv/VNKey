using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Text.Json;
using System.Windows;
using System.Windows.Input;
using VNKey.Windows.Models;
using VNKey.Windows.Infrastructure;
using VNKey.Windows.Services;

using System.IO;
using System.Reflection;

namespace VNKey.Windows.ViewModels
{
    public class MainViewModel : ViewModelBase
    {
        private readonly IEngineService _engineService;
        private readonly IConfigService _configService;
        private readonly IThemeService _themeService;

        private bool _isVietnameseMode;
        private Models.InputMode _currentInputMode;
        private bool? _isDarkMode = null;
        private string _shorthandSearchText = "";
        private string _newMacro = "";
        private string _newExpansion = "";
        private string _currentPage = "Input";
        private bool _isDevModeEnabled;

        // UI Properties
        private bool _isNavVertical = true;
        private bool _isSidebarExpanded = true;

        public bool IsVietnameseMode
        {
            get => _isVietnameseMode;
            set
            {
                if (SetProperty(ref _isVietnameseMode, value))
                {
                    _engineService.SetVietnameseMode(value);
                    
                    // Trigger beep for UI-initiated changes
                    if (BeepOnModeChange)
                    {
                        System.Threading.Tasks.Task.Run(() =>
                        {
                            try
                            {
                                if (value) System.Media.SystemSounds.Asterisk.Play(); // High "Ting" (Vietnamese)
                                else System.Media.SystemSounds.Hand.Play(); // Low "Thud" (English)
                            }
                            catch { }
                        });
                    }
                    
                    SyncAndSave();
                }
            }
        }

        public Models.InputMode CurrentInputMode
        {
            get => _currentInputMode;
            set
            {
                if (SetProperty(ref _currentInputMode, value))
                {
                    _engineService.SetMode(value);
                    SyncAndSave();
                }
            }
        }

        public bool IsDarkMode
        {
            get => _isDarkMode ?? false;
            set
            {
                if (SetProperty(ref _isDarkMode, value))
                {
                    _themeService.IsDarkMode = value;
                    SyncAndSave();
                }
            }
        }

        public bool IsSidebarExpanded
        {
            get => _isSidebarExpanded;
            set => SetProperty(ref _isSidebarExpanded, value);
        }

        public string ShorthandSearchText
        {
            get => _shorthandSearchText;
            set { if (_shorthandSearchText != value) { _shorthandSearchText = value; OnPropertyChanged(); OnPropertyChanged(nameof(FilteredShorthandList)); } }
        }

        public string NewMacro
        {
            get => _newMacro;
            set { if (_newMacro != value) { _newMacro = value; OnPropertyChanged(); } }
        }

        public string NewExpansion
        {
            get => _newExpansion;
            set { if (_newExpansion != value) { _newExpansion = value; OnPropertyChanged(); } }
        }

        private ObservableCollection<Models.ShorthandItem> _shorthandList = new ObservableCollection<Models.ShorthandItem>();
        public ObservableCollection<Models.ShorthandItem> ShorthandList => _shorthandList;

        public ObservableCollection<ShorthandTemplate> BuiltInTemplates { get; } = new ObservableCollection<ShorthandTemplate>();

        public IEnumerable<Models.ShorthandItem> FilteredShorthandList
        {
            get
            {
                if (string.IsNullOrWhiteSpace(_shorthandSearchText)) return _shorthandList;
                var filter = _shorthandSearchText.ToLower();
                return _shorthandList.Where(x => 
                    (x.Macro?.ToLower().Contains(filter) ?? false) || 
                    (x.Expansion?.ToLower().Contains(filter) ?? false));
            }
        }

        public string CurrentPage
        {
            get => _currentPage;
            set => SetProperty(ref _currentPage, value);
        }

        public bool IsDevModeEnabled
        {
            get => _isDevModeEnabled;
            set
            {
                if (SetProperty(ref _isDevModeEnabled, value))
                {
                    SyncAndSave();
                }
            }
        }

        public bool IsNavVertical
        {
            get => _isNavVertical;
            set { if (SetProperty(ref _isNavVertical, value)) SyncAndSave(); }
        }

        // Config properties proxying AppConfig
        public bool ModernTone
        {
            get => _configService.Config.ModernTone;
            set { if (_configService.Config.ModernTone != value) { _configService.Config.ModernTone = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool SpellCheck
        {
            get => _configService.Config.SpellCheck;
            set { if (_configService.Config.SpellCheck != value) { _configService.Config.SpellCheck = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool AutoRestore
        {
            get => _configService.Config.AutoRestore;
            set { if (_configService.Config.AutoRestore != value) { _configService.Config.AutoRestore = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool SmartLiteralMode
        {
            get => _configService.Config.SmartLiteralMode;
            set { if (_configService.Config.SmartLiteralMode != value) { _configService.Config.SmartLiteralMode = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool AllowForeignConsonants
        {
            get => _configService.Config.AllowForeignConsonants;
            set { if (_configService.Config.AllowForeignConsonants != value) { _configService.Config.AllowForeignConsonants = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool AutoCapitalizeSentence
        {
            get => _configService.Config.AutoCapitalizeSentence;
            set { if (_configService.Config.AutoCapitalizeSentence != value) { _configService.Config.AutoCapitalizeSentence = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool AutoCapitalizeEnter
        {
            get => _configService.Config.AutoCapitalizeEnter;
            set { if (_configService.Config.AutoCapitalizeEnter != value) { _configService.Config.AutoCapitalizeEnter = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool BeepOnModeChange
        {
            get => _configService.Config.BeepOnModeChange;
            set { if (_configService.Config.BeepOnModeChange != value) { _configService.Config.BeepOnModeChange = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool PlaySoundOnAction
        {
            get => _configService.Config.PlaySoundOnAction;
            set { if (_configService.Config.PlaySoundOnAction != value) { _configService.Config.PlaySoundOnAction = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool IsRunAtStartup
        {
            get => _configService.Config.StartWithWindows;
            set { if (_configService.Config.StartWithWindows != value) { _configService.Config.StartWithWindows = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public string SwitchShortcut
        {
            get => _configService.Config.SwitchShortcut;
            set { if (_configService.Config.SwitchShortcut != value) { _configService.Config.SwitchShortcut = value; OnPropertyChanged(); SyncAndSave(); } }
        }

        public string CustomShortcut
        {
            get => _configService.Config.CustomShortcut;
            set { if (_configService.Config.CustomShortcut != value) { _configService.Config.CustomShortcut = value; OnPropertyChanged(); SyncAndSave(); } }
        }

        private bool _isRecordingShortcut;
        public bool IsRecordingShortcut
        {
            get => _isRecordingShortcut;
            set => SetProperty(ref _isRecordingShortcut, value);
        }

        public bool ShorthandWhileOff
        {
            get => _configService.Config.ShorthandWhileOff;
            set { if (_configService.Config.ShorthandWhileOff != value) { _configService.Config.ShorthandWhileOff = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public bool ShorthandAutoCase
        {
            get => _configService.Config.ShorthandAutoCase;
            set { if (_configService.Config.ShorthandAutoCase != value) { _configService.Config.ShorthandAutoCase = value; OnPropertyChanged(); PlayToggleSound(); SyncAndSave(); } }
        }

        public DiagnosticsViewModel Diagnostics { get; }

        // Commands
        public ICommand ToggleModeCommand { get; }
        public ICommand NavigateCommand { get; }
        public ICommand ToggleThemeCommand { get; }
        public ICommand ReturnDiagnosticsCommand { get; }
        public ICommand SaveCommand { get; }
        public ICommand RecordShortcutCommand { get; }
        public ICommand ToggleSidebarCommand { get; }
        public ICommand ApplyTemplateCommand { get; }
        public ICommand OpenTemplatesCommand { get; }
        public ICommand RefreshTemplatesCommand { get; }

        public MainViewModel(IEngineService engineService, IConfigService configService, IThemeService themeService, DiagnosticsViewModel diagnostics)
        {
            _engineService = engineService;
            _configService = configService;
            _themeService = themeService;
            Diagnostics = diagnostics;

            // Initialize state from config
            var config = _configService.Config;
            _isVietnameseMode = config.IsVietnameseMode;
            IsDarkMode = config.IsDarkMode ?? _themeService.GetWindowsTheme();
            _currentInputMode = config.CurrentInputMode;
            _isDevModeEnabled = config.IsDevModeEnabled;
            _isNavVertical = config.NavLayout == 0;

            // Initialize Shorthand list
            _shorthandList = new ObservableCollection<Models.ShorthandItem>(config.ShorthandEntries);
            _shorthandList.CollectionChanged += (s, e) => { OnPropertyChanged(nameof(FilteredShorthandList)); OnPropertyChanged(nameof(ShorthandList)); };

            // Commands
            ToggleModeCommand = new RelayCommand(_ => IsVietnameseMode = !IsVietnameseMode);
            NavigateCommand = new RelayCommand(p => CurrentPage = p?.ToString() ?? "Input");
            ToggleThemeCommand = new RelayCommand(_ => IsDarkMode = !IsDarkMode);
            ReturnDiagnosticsCommand = new RelayCommand(_ => Diagnostics.IsPopOut = false);
            SaveCommand = new RelayCommand(_ => _configService.Save());
            ResetCommand = new RelayCommand(_ => ResetConfig());
            RecordShortcutCommand = new RelayCommand(_ => IsRecordingShortcut = !IsRecordingShortcut);
            
            AddShorthandCommand = new RelayCommand(_ => AddShorthand());
            DeleteShorthandCommand = new RelayCommand(p => DeleteShorthand(p as Models.ShorthandItem));
            ImportShorthandCommand = new RelayCommand(_ => ImportShorthand());
            ExportShorthandCommand = new RelayCommand(_ => ExportShorthand());
            ToggleSidebarCommand = new RelayCommand(_ => IsSidebarExpanded = !IsSidebarExpanded);
            ApplyTemplateCommand = new RelayCommand(p => ApplyTemplate(p as ShorthandTemplate));
            OpenTemplatesCommand = new RelayCommand(_ => OpenTemplatesFolder());
            RefreshTemplatesCommand = new RelayCommand(_ => LoadBuiltInTemplates());

            LoadBuiltInTemplates();

            // Subscribe to engine events
            _engineService.OnVietnameseModeChanged += (enabled) => 
            {
                App.Current.Dispatcher.Invoke(() =>
                {
                    if (_isVietnameseMode != enabled)
                    {
                        _isVietnameseMode = enabled;
                        OnPropertyChanged(nameof(IsVietnameseMode));
                        
                        if (BeepOnModeChange)
                        {
                            System.Threading.Tasks.Task.Run(() =>
                            {
                                try
                                {
                                    if (enabled) System.Media.SystemSounds.Asterisk.Play(); // High "Ting" (Vietnamese)
                                    else System.Media.SystemSounds.Hand.Play(); // Low "Thud" (English)
                                }
                                catch { }
                            });
                        }
                    }
                });
            };

            // Subscribe to secret dev mode activation (type "vnkdev" to toggle)
            Diagnostics.DevModeToggleRequested += () =>
            {
                App.Current.Dispatcher.Invoke(() =>
                {
                    IsDevModeEnabled = !IsDevModeEnabled;
                    if (IsDevModeEnabled)
                        CurrentPage = "Diagnostics";
                });
            };

            // Initial apply
            ApplyConfigToEngine();
        }

        private void SyncAndSave()
        {
            var config = _configService.Config;
            config.IsVietnameseMode = _isVietnameseMode;
            config.CurrentInputMode = _currentInputMode;
            config.IsDarkMode = _isDarkMode;
            config.IsDevModeEnabled = _isDevModeEnabled;
            config.NavLayout = _isNavVertical ? 0 : 1;
            _configService.Save();
            
            // Apply to engine
            ApplyConfigToEngine();
        }

        public ICommand ResetCommand { get; }
        public ICommand AddShorthandCommand { get; }
        public ICommand DeleteShorthandCommand { get; }
        public ICommand ImportShorthandCommand { get; }
        public ICommand ExportShorthandCommand { get; }

        private void ApplyTemplate(ShorthandTemplate? template)
        {
            if (template == null) return;
            
            int addedCount = 0;
            foreach (var item in template.Items)
            {
                if (!_shorthandList.Any(x => x.Macro == item.Macro))
                {
                    _shorthandList.Add(new Models.ShorthandItem { Macro = item.Macro, Expansion = item.Expansion });
                    addedCount++;
                }
            }
            
            if (addedCount > 0)
            {
                SyncAndSave();
                System.Windows.MessageBox.Show($"Đã thêm {addedCount} mục gõ tắt từ mẫu '{template.Name}'.");
            }
            else
            {
                System.Windows.MessageBox.Show("Các mục trong mẫu này đã tồn tại trong danh sách của bạn.");
            }
        }

        private void ResetConfig()
        {
            if (System.Windows.MessageBox.Show("Bạn có chắc chắn muốn khôi phục cài đặt mặc định?", "Xác nhận", MessageBoxButton.YesNo, MessageBoxImage.Question) == MessageBoxResult.Yes)
            {
                _configService.ResetToDefault();
                OnPropertyChanged(null); // Notify all properties changed
                ApplyConfigToEngine();
                SyncAndSave();
            }
        }

        private void AddShorthand()
        {
            if (string.IsNullOrWhiteSpace(NewMacro) || string.IsNullOrWhiteSpace(NewExpansion)) return;
            
            var existing = _shorthandList.FirstOrDefault(x => x.Macro == NewMacro);
            if (existing != null)
            {
                existing.Expansion = NewExpansion;
                OnPropertyChanged(nameof(FilteredShorthandList));
            }
            else
            {
                _shorthandList.Add(new Models.ShorthandItem { Macro = NewMacro, Expansion = NewExpansion });
            }

            NewMacro = "";
            NewExpansion = "";
            SyncAndSave();
        }

        private void DeleteShorthand(Models.ShorthandItem? item)
        {
            if (item == null) return;
            _shorthandList.Remove(item);
            SyncAndSave();
        }

        private void ImportShorthand()
        {
            var openFileDialog = new Microsoft.Win32.OpenFileDialog();
            openFileDialog.Filter = "Text files (*.txt)|*.txt|JSON files (*.json)|*.json|All files (*.*)|*.*";
            if (openFileDialog.ShowDialog() == true)
            {
                try
                {
                    string content = System.IO.File.ReadAllText(openFileDialog.FileName);
                    if (openFileDialog.FileName.EndsWith(".json"))
                    {
                        var imported = JsonSerializer.Deserialize<Dictionary<string, string>>(content);
                        if (imported != null)
                        {
                            foreach (var kv in imported)
                            {
                                if (!_shorthandList.Any(x => x.Macro == kv.Key))
                                    _shorthandList.Add(new Models.ShorthandItem { Macro = kv.Key, Expansion = kv.Value });
                            }
                        }
                    }
                    else
                    {
                        foreach (var line in content.Split(new[] { '\n', '\r' }, StringSplitOptions.RemoveEmptyEntries))
                        {
                            var parts = line.Split(new[] { ':', '\t' }, 2);
                            if (parts.Length == 2)
                            {
                                string m = parts[0].Trim();
                                string exp = parts[1].Trim();
                                if (!string.IsNullOrEmpty(m) && !_shorthandList.Any(x => x.Macro == m))
                                    _shorthandList.Add(new Models.ShorthandItem { Macro = m, Expansion = exp });
                            }
                        }
                    }
                    SyncAndSave();
                }
                catch (Exception ex)
                {
                    System.Windows.MessageBox.Show("Lỗi khi nhập file: " + ex.Message);
                }
            }
        }

        private void ExportShorthand()
        {
            var saveFileDialog = new Microsoft.Win32.SaveFileDialog();
            saveFileDialog.Filter = "Text files (*.txt)|*.txt|JSON files (*.json)|*.json";
            if (saveFileDialog.ShowDialog() == true)
            {
                try
                {
                    if (saveFileDialog.FileName.EndsWith(".json"))
                    {
                        var dict = _shorthandList.ToDictionary(x => x.Macro, x => x.Expansion);
                        string json = JsonSerializer.Serialize(dict, new JsonSerializerOptions { WriteIndented = true });
                        System.IO.File.WriteAllText(saveFileDialog.FileName, json);
                    }
                    else
                    {
                        var lines = _shorthandList.Select(x => $"{x.Macro}:{x.Expansion}");
                        System.IO.File.WriteAllLines(saveFileDialog.FileName, lines);
                    }
                }
                catch (Exception ex)
                {
                    System.Windows.MessageBox.Show("Lỗi khi xuất file: " + ex.Message);
                }
            }
        }

        private void OpenTemplatesFolder()
        {
            try
            {
                string templatesPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "VNKey", "ShorthandTemplates.json");
                
                // If it doesn't exist in AppData yet, create the directory and copy the default
                if (!File.Exists(templatesPath))
                {
                    string appDataDir = Path.GetDirectoryName(templatesPath)!;
                    if (!Directory.Exists(appDataDir)) Directory.CreateDirectory(appDataDir);
                    
                    string defaultTemplatesPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "ShorthandTemplates.json");
                    if (File.Exists(defaultTemplatesPath))
                    {
                        File.Copy(defaultTemplatesPath, templatesPath);
                    }
                }

                if (File.Exists(templatesPath))
                {
                    // Open the file with the default text editor
                    System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
                    {
                        FileName = templatesPath,
                        UseShellExecute = true
                    });
                }
            }
            catch (Exception ex)
            {
                System.Windows.MessageBox.Show($"Lỗi khi mở file mẫu: {ex.Message}", "Lỗi", System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Error);
            }
        }

        private void LoadBuiltInTemplates()
        {
            try
            {
                BuiltInTemplates.Clear();
                string appDataDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "VNKey");
                if (!Directory.Exists(appDataDir)) Directory.CreateDirectory(appDataDir);
                
                string templatesPath = Path.Combine(appDataDir, "ShorthandTemplates.json");
                string defaultTemplatesPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "ShorthandTemplates.json");

                if (!File.Exists(templatesPath) && File.Exists(defaultTemplatesPath))
                {
                    File.Copy(defaultTemplatesPath, templatesPath);
                }

                if (File.Exists(templatesPath))
                {
                    string json = File.ReadAllText(templatesPath);
                    var templates = JsonSerializer.Deserialize<List<ShorthandTemplate>>(json);
                    if (templates != null)
                    {
                        foreach (var template in templates)
                        {
                            BuiltInTemplates.Add(template);
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                System.Windows.MessageBox.Show($"Lỗi khi nạp danh sách mẫu: {ex.Message}", "Lỗi", System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Error);
            }
        }

        private void ApplyConfigToEngine()
        {
            var config = _configService.Config;
            config.ShorthandEntries = _shorthandList.ToList();

            var rustConfig = new Dictionary<string, object>
            {
                { "vietnamese_mode", _isVietnameseMode },
                { "modern_tone", config.ModernTone },
                { "free_typing", config.FreeMarking },
                { "spell_check", config.SpellCheck },
                { "auto_restore", config.AutoRestore },
                { "smart_literal_mode", config.SmartLiteralMode },
                { "allow_foreign_consonants", config.AllowForeignConsonants },
                { "auto_capitalize_sentence", config.AutoCapitalizeSentence },
                { "auto_capitalize_enter", config.AutoCapitalizeEnter },
                { "shorthand_while_off", config.ShorthandWhileOff },
                { "macro_auto_case", config.ShorthandAutoCase },
                { "macro_enabled", true }, // Always enable at engine level if configured
                { "beep_on_switch", config.BeepOnModeChange },
                { "output_charset", "Unicode" },
                { "switch_shortcut", config.SwitchShortcut == "Custom" ? config.CustomShortcut : config.SwitchShortcut },
                { "simulation_delay", (uint)config.SimulationDelay },
                { "backspace_restore", true },
                { "show_feedback", true }
            };
            string json = JsonSerializer.Serialize(rustConfig);
            _engineService.SetConfig(json);

            // Shorthand entries - ONLY sync if there are entries to avoid wiping defaults
            // if the user hasn't configured any yet but wants the built-ins.
            if (config.ShorthandEntries.Any())
            {
                var shorthandMap = config.ShorthandEntries
                    .Where(x => !string.IsNullOrWhiteSpace(x.Macro))
                    .ToDictionary(x => x.Macro, x => x.Expansion ?? "");
                string jsonShorthand = JsonSerializer.Serialize(shorthandMap);
                _engineService.SetShorthand(jsonShorthand);
            }
        }

        private void PlayToggleSound()
        {
            if (PlaySoundOnAction)
            {
                System.Threading.Tasks.Task.Run(() =>
                {
                    try
                    {
                        System.Media.SystemSounds.Exclamation.Play(); // Snappy feedback
                    }
                    catch { }
                });
            }
        }
    }
}

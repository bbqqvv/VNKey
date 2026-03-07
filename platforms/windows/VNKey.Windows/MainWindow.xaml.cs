using System;
using System.Linq;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Input;
using System.Windows.Data;
using System.ComponentModel;
using VNKey.Windows.Core;
using Microsoft.Win32;

namespace VNKey.Windows
{
    public partial class MainWindow : System.Windows.Window
    {
        private EngineWrapper _engine;
        private KeyboardHook _hook;
        private System.Windows.Forms.NotifyIcon _notifyIcon;
        private bool _isRecordingHotkey = false;

        // ObservableCollection cho DataGrid Gõ tắt
        public ObservableCollection<ShorthandItem> ShorthandList { get; set; }

        public MainWindow()
        {
            try 
            {
                InitializeComponent();
                _engine = new EngineWrapper();
                _hook = new KeyboardHook(_engine);
                
                // Load Vietnamese dictionary
                string baseDir = AppDomain.CurrentDomain.BaseDirectory;
                string dicPath = System.IO.Path.Combine(baseDir, "Native", "dictionaries", "vi.dic");
                if (System.IO.File.Exists(dicPath))
                {
                    EngineWrapper.vnkey_global_load_dictionary(dicPath);
                }
                
                // Subscribe to mode changes (e.g. from Alt+Z)
                _hook.OnVietnameseModeChanged += UpdateStatusUi;

                InitializeTrayIcon();

                // Fake Data Gõ tắt
                ShorthandList = new ObservableCollection<ShorthandItem>
                {
                    new ShorthandItem { Macro = "ko", Expansion = "không" },
                    new ShorthandItem { Macro = "dc", Expansion = "được" },
                    new ShorthandItem { Macro = "vn", Expansion = "Việt Nam" }
                };
                // DataGridShorthand.ItemsSource = ShorthandList; // Will be added in shorthand tab if needed
                
                // Load cấu hình lên UI
                LoadConfigToUi();
                
                // Đăng ký sự kiện thay đổi cho tất cả control để tự động lưu & áp dụng
                RegisterConfigChangeEvents();
            }
            catch (Exception ex)
            {
                System.Windows.MessageBox.Show($"Lỗi khởi tạo: {ex.Message}\n{ex.StackTrace}", "VNKey Error", System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Error);
                System.Windows.Application.Current.Shutdown();
            }
        }

        private void UpdateStatusUi(bool isEnabled)
        {
            Dispatcher.Invoke(() =>
            {
                // Sync Radio Buttons
                if (isEnabled)
                {
                    RadVietnamese.IsChecked = true;
                }
                else
                {
                    RadEnglish.IsChecked = true;
                }

                // Update Tray Icon
                var iconUri = isEnabled 
                    ? new Uri("pack://application:,,,/Assets/V.png") 
                    : new Uri("pack://application:,,,/Assets/E.png");

                try
                {
                    var streamInfo = System.Windows.Application.GetResourceStream(iconUri);
                    if (streamInfo != null)
                    {
                        using (var stream = streamInfo.Stream)
                        {
                            using (var bmp = new System.Drawing.Bitmap(stream))
                            {
                                var hIcon = bmp.GetHicon();
                                _notifyIcon.Icon = System.Drawing.Icon.FromHandle(hIcon);
                            }
                        }
                    }
                }
                catch { /* Ignore icon update errors */ }

                if (!_isFirstLoad && App.Config.BeepOnSwitch)
                {
                    if (isEnabled)
                        System.Console.Beep(800, 100);
                    else
                        System.Console.Beep(400, 100);
                }

                // Luôn đồng bộ trạng thái E/V vào biến toàn cục để tránh bị ghi đè khi lưu Setting khác
                App.Config.IsVietnameseMode = isEnabled;
            });
        }

        private void ComboInputMethod_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (_engine == null) return;
            var comboBox = sender as System.Windows.Controls.ComboBox;
            if (comboBox == null) return;

            VNKey.Windows.Core.InputMode mode = (VNKey.Windows.Core.InputMode)(byte)comboBox.SelectedIndex;
            _engine.SetMode(mode);
            SyncAndSave();
        }

        private void TitleBar_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            this.DragMove();
        }

        private void InitializeTrayIcon()
        {
            _notifyIcon = new System.Windows.Forms.NotifyIcon();
            
            try {
                var iconUri = new Uri("pack://application:,,,/Assets/VNKey.png");
                var streamInfo = System.Windows.Application.GetResourceStream(iconUri);
                if (streamInfo != null) {
                    using (var stream = streamInfo.Stream) {
                        using (var bmp = new System.Drawing.Bitmap(stream)) {
                            var hIcon = bmp.GetHicon();
                            _notifyIcon.Icon = System.Drawing.Icon.FromHandle(hIcon);
                        }
                    }
                }
            } catch {
                _notifyIcon.Icon = System.Drawing.Icon.ExtractAssociatedIcon(System.Windows.Forms.Application.ExecutablePath);
            }

            _notifyIcon.Visible = true;
            _notifyIcon.Text = "VNKey 2.0";
            _notifyIcon.DoubleClick += (s, e) => ShowWindow();

            var contextMenu = new System.Windows.Forms.ContextMenuStrip();
            contextMenu.Items.Add("Hiện bảng biểu khiển", null, (s, e) => ShowWindow());
            contextMenu.Items.Add("Thoát", null, (s, e) => ExitButton_Click(s, null!));
            _notifyIcon.ContextMenuStrip = contextMenu;
        }

        private void LoadConfigToUi()
        {
            var config = App.Config;
            
            ComboInputMethod.SelectedIndex = (int)config.CurrentInputMode;
            
            RadVietnamese.IsChecked = config.IsVietnameseMode;
            RadEnglish.IsChecked = !config.IsVietnameseMode;

            ChkModernTone.IsChecked = config.ModernTone;
            ChkSpellCheck.IsChecked = config.SpellCheck;
            ChkAutoRestore.IsChecked = config.AutoRestore;
            ChkSmartLiteralMode.IsChecked = config.SmartLiteralMode;
            ChkAllowForeign.IsChecked = config.AllowForeignConsonants;
            // Removed SliderDelay load

            ChkAutoCapitalizeSentence.IsChecked = config.AutoCapitalizeSentence;
            ChkAutoCapitalizeEnter.IsChecked = config.AutoCapitalizeEnter;

            // Tab Gõ tắt
            ChkShorthandWhileOff.IsChecked = config.ShorthandWhileOff;
            ChkShorthandAutoCase.IsChecked = config.ShorthandAutoCase;

            // Load shorthand list
            ShorthandList.Clear();
            foreach (var item in config.ShorthandEntries)
            {
                ShorthandList.Add(item);
            }

            // Tab Hệ thống
            ChkRunAtStartup.IsChecked = config.StartWithWindows;
            if (config.SwitchShortcut == "Ctrl+Shift") RbHotkeyCtrlShift.IsChecked = true;
            else if (config.SwitchShortcut == "Alt+Z") RbHotkeyAltZ.IsChecked = true;
            else
            {
                RbHotkeyCustom.IsChecked = true;
                BtnRecordHotkey.Content = config.SwitchShortcut;
            }

            ChkBeep.IsChecked = config.BeepOnSwitch;
            _sidebarState = config.NavLayout;
            if (_sidebarState == 2) RbNavHorizontal.IsChecked = true;
            else RbNavVertical.IsChecked = true;

            UpdateNavigationLayout();
            ApplyConfigToEngine();

            // Theme initialization: Default to OS theme if not set
            if (config.IsDarkMode == null)
            {
                _isDarkMode = GetWindowsTheme();
            }
            else
            {
                _isDarkMode = config.IsDarkMode.Value;
            }
            ApplyTheme();

            _isFirstLoad = false;
        }

        private void RegisterConfigChangeEvents()
        {
            ChkModernTone.Click += (s, e) => SyncAndSave();
            ChkSpellCheck.Click += (s, e) => SyncAndSave();
            ChkAutoRestore.Click += (s, e) => SyncAndSave();
            ChkSmartLiteralMode.Click += (s, e) => SyncAndSave();
            ChkAllowForeign.Click += (s, e) => SyncAndSave();
            ChkAutoCapitalizeSentence.Click += (s, e) => SyncAndSave();
            ChkAutoCapitalizeEnter.Click += (s, e) => SyncAndSave();

            RadVietnamese.Click += (s, e) => SyncAndSave();
            RadEnglish.Click += (s, e) => SyncAndSave();
            
            ChkRunAtStartup.Click += (s, e) => SyncAndSave();

            ChkShorthandWhileOff.Click += (s, e) => SyncAndSave();
            ChkShorthandAutoCase.Click += (s, e) => SyncAndSave();
            ChkBeep.Click += (s, e) => SyncAndSave();
            RbNavVertical.Click += (s, e) => SyncAndSave();
            RbNavHorizontal.Click += (s, e) => SyncAndSave();
            RbHotkeyAltZ.Click += (s, e) => SyncAndSave();
            RbHotkeyCtrlShift.Click += (s, e) => SyncAndSave();
            RbHotkeyCustom.Click += (s, e) => SyncAndSave();

            ShorthandList.CollectionChanged += (s, e) => SyncAndSave();
        }

        private void SyncAndSave()
        {
            SaveUiToConfig();
            App.Config.Save();
            ApplyConfigToEngine();
        }

        private void SaveUiToConfig()
        {
            var config = App.Config;
            config.CurrentInputMode = (VNKey.Windows.Core.InputMode)ComboInputMethod.SelectedIndex;
            config.IsVietnameseMode = RadVietnamese.IsChecked ?? true;
            
            config.ModernTone = ChkModernTone.IsChecked ?? true;
            config.SpellCheck = ChkSpellCheck.IsChecked ?? false;
            config.AutoRestore = ChkAutoRestore.IsChecked ?? false;
            config.SmartLiteralMode = ChkSmartLiteralMode.IsChecked ?? true;
            config.AllowForeignConsonants = ChkAllowForeign.IsChecked ?? true;
            config.AutoCapitalizeSentence = ChkAutoCapitalizeSentence.IsChecked ?? false;
            config.AutoCapitalizeEnter = ChkAutoCapitalizeEnter.IsChecked ?? false;
            
            config.StartWithWindows = ChkRunAtStartup.IsChecked ?? false;
            config.BeepOnSwitch = ChkBeep.IsChecked ?? false;
            
            string currentShortcut = BtnRecordHotkey.Content.ToString();
            if (currentShortcut != "Record" && currentShortcut != "...")
            {
                config.SwitchShortcut = RbHotkeyCtrlShift.IsChecked == true ? "Ctrl+Shift" : 
                                        (RbHotkeyAltZ.IsChecked == true ? "Alt+Z" : currentShortcut);
            }
            else if (RbHotkeyCtrlShift.IsChecked == true) config.SwitchShortcut = "Ctrl+Shift";
            else if (RbHotkeyAltZ.IsChecked == true) config.SwitchShortcut = "Alt+Z";

            config.NavLayout = _sidebarState;
            // Removed SimulationDelay save

            config.ShorthandWhileOff = ChkShorthandWhileOff.IsChecked ?? false;
            config.ShorthandAutoCase = ChkShorthandAutoCase.IsChecked ?? true;

            config.IsDarkMode = _isDarkMode;
            config.ShorthandEntries = ShorthandList.ToList();
        }

        private void ApplyConfigToEngine()
        {
            if (_engine == null) return;

            var rustConfig = new Dictionary<string, object>
            {
                { "vietnamese_mode", App.Config.IsVietnameseMode },
                { "free_typing", App.Config.FreeMarking },
                { "modern_tone", App.Config.ModernTone },
                { "spell_check", App.Config.SpellCheck },
                { "auto_restore", App.Config.AutoRestore },
                { "smart_literal_mode", App.Config.SmartLiteralMode },
                { "allow_foreign_consonants", App.Config.AllowForeignConsonants },
                { "auto_capitalize_sentence", App.Config.AutoCapitalizeSentence },
                { "auto_capitalize_enter", App.Config.AutoCapitalizeEnter },
                { "macro_enabled", ShorthandList.Count > 0 },
                { "shorthand_while_off", App.Config.ShorthandWhileOff },
                { "macro_auto_case", App.Config.ShorthandAutoCase },
                { "output_charset", "Unicode" },
                { "switch_shortcut", App.Config.SwitchShortcut }
            };

            string jsonConfig = System.Text.Json.JsonSerializer.Serialize(rustConfig);
            EngineWrapper.vnkey_global_set_config_json(jsonConfig);

            var shorthandMap = ShorthandList.ToDictionary(x => x.Macro, x => x.Expansion);
            string jsonShorthand = System.Text.Json.JsonSerializer.Serialize(shorthandMap);
            EngineWrapper.vnkey_global_set_shorthand_json(jsonShorthand);

            UpdateStatusUi(App.Config.IsVietnameseMode);
        }

        private void ShowWindow()
        {
            this.Show();
            this.WindowState = WindowState.Normal;
            this.Activate();
            this.Topmost = true;
            this.Topmost = false;
        }

        private void RbHotkey_Click(object sender, RoutedEventArgs e)
        {
            if (sender == RbHotkeyAltZ) BtnRecordHotkey.Content = "Record";
            if (sender == RbHotkeyCtrlShift) BtnRecordHotkey.Content = "Record";
            SyncAndSave();
        }

        private void BtnRecordHotkey_Click(object sender, RoutedEventArgs e)
        {
            _isRecordingHotkey = true;
            BtnRecordHotkey.Content = "...";
            BtnRecordHotkey.Focus();
        }

        protected override void OnKeyDown(System.Windows.Input.KeyEventArgs e)
        {
            if (_isRecordingHotkey)
            {
                e.Handled = true;
                var key = e.Key == System.Windows.Input.Key.System ? e.SystemKey : e.Key;

                // Ignore pure modifiers
                if (key == System.Windows.Input.Key.LeftCtrl || key == System.Windows.Input.Key.RightCtrl ||
                    key == System.Windows.Input.Key.LeftAlt || key == System.Windows.Input.Key.RightAlt ||
                    key == System.Windows.Input.Key.LeftShift || key == System.Windows.Input.Key.RightShift ||
                    key == System.Windows.Input.Key.LWin || key == System.Windows.Input.Key.RWin)
                {
                    return;
                }

                string combo = "";
                if ((System.Windows.Input.Keyboard.Modifiers & System.Windows.Input.ModifierKeys.Control) != 0) combo += "Ctrl+";
                if ((System.Windows.Input.Keyboard.Modifiers & System.Windows.Input.ModifierKeys.Alt) != 0) combo += "Alt+";
                if ((System.Windows.Input.Keyboard.Modifiers & System.Windows.Input.ModifierKeys.Shift) != 0) combo += "Shift+";
                if ((System.Windows.Input.Keyboard.Modifiers & System.Windows.Input.ModifierKeys.Windows) != 0) combo += "Win+";

                combo += key.ToString();
                BtnRecordHotkey.Content = combo;
                _isRecordingHotkey = false;
                SyncAndSave();
                return;
            }
            base.OnKeyDown(e);
        }

        // ═══ SHORTHAND LOGIC ═══

        private void TxtSearchShorthand_TextChanged(object sender, TextChangedEventArgs e)
        {
            if (ShorthandList == null) return;
            string filter = TxtSearchShorthand.Text.ToLower();
            var view = CollectionViewSource.GetDefaultView(ShorthandList);
            if (string.IsNullOrEmpty(filter))
            {
                view.Filter = null;
            }
            else
            {
                view.Filter = item =>
                {
                    var si = item as ShorthandItem;
                    if (si == null) return false;
                    return (si.Macro != null && si.Macro.ToLower().Contains(filter)) || 
                           (si.Expansion != null && si.Expansion.ToLower().Contains(filter));
                };
            }
        }

        private void BtnAddShorthand_Click(object sender, RoutedEventArgs e)
        {
            string macro = TxtMacro.Text.Trim();
            string expansion = TxtExpansion.Text.Trim();

            if (string.IsNullOrEmpty(macro) || string.IsNullOrEmpty(expansion))
                return;

            var existing = ShorthandList.FirstOrDefault(x => x.Macro == macro);
            if (existing != null)
            {
                existing.Expansion = expansion;
                // Re-insert to trigger UI update
                int index = ShorthandList.IndexOf(existing);
                ShorthandList.RemoveAt(index);
                ShorthandList.Insert(index, existing);
            }
            else
            {
                ShorthandList.Add(new ShorthandItem { Macro = macro, Expansion = expansion });
            }

            TxtMacro.Clear();
            TxtExpansion.Clear();
            TxtMacro.Focus();
        }

        private void BtnDeleteShorthand_Click(object sender, RoutedEventArgs e)
        {
            var btn = sender as System.Windows.Controls.Button;
            var item = btn?.DataContext as ShorthandItem;
            if (item != null)
            {
                ShorthandList.Remove(item);
            }
        }

        private void BtnImport_Click(object sender, RoutedEventArgs e)
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
                        var imported = System.Text.Json.JsonSerializer.Deserialize<Dictionary<string, string>>(content);
                        if (imported != null)
                        {
                            foreach (var kv in imported)
                            {
                                if (!ShorthandList.Any(x => x.Macro == kv.Key))
                                    ShorthandList.Add(new ShorthandItem { Macro = kv.Key, Expansion = kv.Value });
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
                                if (!string.IsNullOrEmpty(m) && !ShorthandList.Any(x => x.Macro == m))
                                    ShorthandList.Add(new ShorthandItem { Macro = m, Expansion = exp });
                            }
                        }
                    }
                }
                catch (Exception ex)
                {
                    System.Windows.MessageBox.Show("Lỗi khi nhập file: " + ex.Message);
                }
            }
        }

        private void BtnExport_Click(object sender, RoutedEventArgs e)
        {
            var saveFileDialog = new Microsoft.Win32.SaveFileDialog();
            saveFileDialog.Filter = "Text files (*.txt)|*.txt|JSON files (*.json)|*.json";
            if (saveFileDialog.ShowDialog() == true)
            {
                try
                {
                    if (saveFileDialog.FileName.EndsWith(".json"))
                    {
                        var dict = ShorthandList.ToDictionary(x => x.Macro, x => x.Expansion);
                        string json = System.Text.Json.JsonSerializer.Serialize(dict, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
                        System.IO.File.WriteAllText(saveFileDialog.FileName, json);
                    }
                    else
                    {
                        var lines = ShorthandList.Select(x => $"{x.Macro}:{x.Expansion}");
                        System.IO.File.WriteAllLines(saveFileDialog.FileName, lines);
                    }
                }
                catch (Exception ex)
                {
                    System.Windows.MessageBox.Show("Lỗi khi xuất file: " + ex.Message);
                }
            }
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            this.Hide(); 
        }

        private void BtnDefault_Click(object sender, RoutedEventArgs e)
        {
            if (System.Windows.MessageBox.Show("Bạn có chắc chắn muốn khôi phục cài đặt mặc định?", "Xác nhận", System.Windows.MessageBoxButton.YesNo, System.Windows.MessageBoxImage.Question) == System.Windows.MessageBoxResult.Yes)
            {
                App.Config = new VNKey.Windows.Core.AppConfig();
                LoadConfigToUi();
                SyncAndSave();
            }
        }

        private void ExitButton_Click(object sender, RoutedEventArgs e)
        {
            Cleanup();
            System.Windows.Application.Current.Shutdown();
        }

        private void Cleanup()
        {
            _notifyIcon?.Dispose();
            _hook?.Dispose();
            _engine?.Dispose();
        }

        private bool _isDarkMode = true;

        private void ThemeToggle_Click(object sender, RoutedEventArgs e)
        {
            _isDarkMode = !_isDarkMode;
            ApplyTheme();
            SaveUiToConfig();
        }

        private void ApplyTheme()
        {
            if (_isDarkMode)
            {
                this.Resources["AppBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#1E1E1E"));
                this.Resources["AppControlBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#252526"));
                this.Resources["AppAccent"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#00E5FF"));
                this.Resources["AppBorder"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#3E3E42"));
                this.Resources["AppText"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#FFFFFF"));
                this.Resources["AppSecondaryText"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#969696"));
                
                this.Resources["AppButtonBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#333333"));
                this.Resources["AppButtonHoverBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#3F3F46"));
                this.Resources["AppAccentForeground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#1E1E1E"));
                this.Resources["AppAccentHoverBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#80F2FF"));
                this.Resources["AppSidebarBackground"] = System.Windows.Media.Brushes.Transparent;
                this.Resources["AppCardBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#2D2D30"));
                this.Resources["AppScrollbarThumb"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#00E5FF"));
                
                // Bổ sung cho TextBox và DataGrid
                this.Resources["AppInputBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#1E1E1E"));
                this.Resources["AppInputForeground"] = System.Windows.Media.Brushes.White;
                this.Resources["AppAlternatingRow"] = new SolidColorBrush(System.Windows.Media.Color.FromArgb(12, 255, 255, 255)); // #0CFFFFFF
                
                if (this.FindName("ThemeIcon") is TextBlock themeIcon)
                {
                    themeIcon.Text = "\xE706"; // Brightness/Sun icon
                }
            }
            else
            {
                this.Resources["AppBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#F3F3F3"));
                this.Resources["AppControlBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#FFFFFF"));
                this.Resources["AppAccent"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#0078D7"));
                this.Resources["AppBorder"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#CCCCCC"));
                this.Resources["AppText"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#000000"));
                this.Resources["AppSecondaryText"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#666666"));

                this.Resources["AppButtonBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#E1E1E1"));
                this.Resources["AppButtonHoverBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#D0D0D0"));
                this.Resources["AppAccentForeground"] = new SolidColorBrush(System.Windows.Media.Colors.White);
                this.Resources["AppAccentHoverBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#005A9E"));
                this.Resources["AppSidebarBackground"] = System.Windows.Media.Brushes.Transparent;
                this.Resources["AppCardBackground"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#FFFFFF"));
                this.Resources["AppScrollbarThumb"] = new SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#AAAAAA"));

                // Bổ sung cho TextBox và DataGrid
                this.Resources["AppInputBackground"] = System.Windows.Media.Brushes.White;
                this.Resources["AppInputForeground"] = System.Windows.Media.Brushes.Black;
                this.Resources["AppAlternatingRow"] = new SolidColorBrush(System.Windows.Media.Color.FromArgb(12, 0, 0, 0)); // #0C000000

                if (this.FindName("ThemeIcon") is TextBlock themeIcon)
                {
                    themeIcon.Text = "\xE708"; // Moon/Dark Mode icon
                }
            }
        }

        private bool GetWindowsTheme()
        {
            try
            {
                using (var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"))
                {
                    if (key != null)
                    {
                        object registryValueObject = key.GetValue("AppsUseLightTheme");
                        if (registryValueObject != null)
                        {
                            return (int)registryValueObject == 0; // 0 = Dark, 1 = Light
                        }
                    }
                }
            }
            catch { }
            return true; // Mặc định là Dark nếu không tìm thấy hoặc lỗi Registry
        }

        private int _sidebarState = 0; // 0 = Expanded Vertical, 1 = Collapsed Vertical, 2 = Horizontal
        private bool _isFirstLoad = true;

        private void Logo_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            e.Handled = true; // Prevent window drag when clicking logo
            
            if (_sidebarState == 2) return; // Do nothing if horizontal

            _sidebarState = _sidebarState == 0 ? 1 : 0;
            UpdateNavigationLayout();
        }

        private void UINavLayout_Checked(object sender, RoutedEventArgs e)
        {
            if (HorizontalNavBorder == null || SidebarBorder == null || RbNavHorizontal == null) return; // Ignore during initialization

            if (RbNavHorizontal.IsChecked == true)
            {
                _sidebarState = 2; // Horizontal
            }
            else
            {
                // Restore vertical, default to expanded
                _sidebarState = 0; 
            }
            UpdateNavigationLayout();
        }

        private void UpdateNavigationLayout()
        {
            if (_sidebarState == 0) // Expanded Vertical
            {
                if (HorizontalNavBorder.Child != null)
                {
                    HorizontalNavBorder.Child = null;
                    SidebarBorder.Child = NavButtonsStack;
                }
                NavButtonsStack.Orientation = System.Windows.Controls.Orientation.Vertical;
                NavButtonsStack.Margin = new Thickness(0, 5, 0, 5);
                
                NavInput.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;
                NavShorthand.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;
                NavSystem.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;
                NavAbout.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;

                NavInput.Padding = new Thickness(12, 10, 12, 10);
                NavShorthand.Padding = new Thickness(12, 10, 12, 10);
                NavSystem.Padding = new Thickness(12, 10, 12, 10);
                NavAbout.Padding = new Thickness(12, 10, 12, 10);
                NavButtonsStack.HorizontalAlignment = System.Windows.HorizontalAlignment.Stretch;

                RowHorizontalNav.Height = new GridLength(0);
                ColSidebar.Width = new GridLength(140);
                HorizontalNavBorder.Visibility = Visibility.Collapsed;
                SidebarBorder.Visibility = Visibility.Visible;

                SideModeIndicator.Visibility = Visibility.Visible;
                NavInputText.Visibility = Visibility.Visible;
                NavShorthandText.Visibility = Visibility.Visible;
                NavSystemText.Visibility = Visibility.Visible;
                NavAboutText.Visibility = Visibility.Visible;
            }
            else if (_sidebarState == 1) // Collapsed Vertical
            {
                if (HorizontalNavBorder.Child != null)
                {
                    HorizontalNavBorder.Child = null;
                    SidebarBorder.Child = NavButtonsStack;
                }
                NavButtonsStack.Orientation = System.Windows.Controls.Orientation.Vertical;
                NavButtonsStack.Margin = new Thickness(0, 5, 0, 5);

                // Center icons properly in collapsed state
                NavInput.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Center;
                NavShorthand.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Center;
                NavSystem.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Center;
                NavAbout.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Center;

                NavInput.Padding = new Thickness(0, 10, 0, 10);
                NavShorthand.Padding = new Thickness(0, 10, 0, 10);
                NavSystem.Padding = new Thickness(0, 10, 0, 10);
                NavAbout.Padding = new Thickness(0, 10, 0, 10);

                NavButtonsStack.HorizontalAlignment = System.Windows.HorizontalAlignment.Stretch;

                RowHorizontalNav.Height = new GridLength(0);
                ColSidebar.Width = new GridLength(50);
                HorizontalNavBorder.Visibility = Visibility.Collapsed;
                SidebarBorder.Visibility = Visibility.Visible;

                SideModeIndicator.Visibility = Visibility.Collapsed;
                NavInputText.Visibility = Visibility.Collapsed;
                NavShorthandText.Visibility = Visibility.Collapsed;
                NavSystemText.Visibility = Visibility.Collapsed;
                NavAboutText.Visibility = Visibility.Collapsed;
            }
            else if (_sidebarState == 2) // Horizontal
            {
                if (SidebarBorder.Child != null)
                {
                    SidebarBorder.Child = null;
                    HorizontalNavBorder.Child = NavButtonsStack;
                }
                NavButtonsStack.Orientation = System.Windows.Controls.Orientation.Horizontal;
                NavButtonsStack.Margin = new Thickness(0);
                NavButtonsStack.HorizontalAlignment = System.Windows.HorizontalAlignment.Center;

                NavInput.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;
                NavShorthand.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;
                NavSystem.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;
                NavAbout.HorizontalContentAlignment = System.Windows.HorizontalAlignment.Left;

                NavInput.Padding = new Thickness(15, 5, 15, 5);
                NavShorthand.Padding = new Thickness(15, 5, 15, 5);
                NavSystem.Padding = new Thickness(15, 5, 15, 5);
                NavAbout.Padding = new Thickness(15, 5, 15, 5);

                RowHorizontalNav.Height = new GridLength(34);
                ColSidebar.Width = new GridLength(0);
                HorizontalNavBorder.Visibility = Visibility.Visible;
                SidebarBorder.Visibility = Visibility.Collapsed;

                SideModeIndicator.Visibility = Visibility.Collapsed;
                NavInputText.Visibility = Visibility.Visible;
                NavShorthandText.Visibility = Visibility.Visible;
                NavSystemText.Visibility = Visibility.Visible;
                NavAboutText.Visibility = Visibility.Visible;
            }
        }

        private void Nav_Checked(object sender, RoutedEventArgs e)
        {
            // Guard: pages may not exist yet during InitializeComponent
            if (PageInput == null || PageShorthand == null || PageSystem == null || PageAbout == null)
                return;

            // Hide all pages
            PageInput.Visibility = Visibility.Collapsed;
            PageShorthand.Visibility = Visibility.Collapsed;
            PageSystem.Visibility = Visibility.Collapsed;
            PageAbout.Visibility = Visibility.Collapsed;

            // Show selected page
            if (sender == NavInput) PageInput.Visibility = Visibility.Visible;
            else if (sender == NavShorthand) PageShorthand.Visibility = Visibility.Visible;
            else if (sender == NavSystem) PageSystem.Visibility = Visibility.Visible;
            else if (sender == NavAbout) PageAbout.Visibility = Visibility.Visible;
        }

        protected override void OnClosed(EventArgs e)
        {
            Cleanup();
            base.OnClosed(e);
        }
    }
}

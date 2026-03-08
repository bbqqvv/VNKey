using System;
using System.Windows;
using System.Windows.Input;
using System.Runtime.InteropServices;
using VNKey.Windows.Models;
using VNKey.Windows.ViewModels;

namespace VNKey.Windows.Views
{
    public partial class MainWindow : Window
    {
        private readonly MainViewModel _viewModel;
        private System.Windows.Forms.NotifyIcon _notifyIcon;
        private System.Drawing.Icon? _iconV;
        private System.Drawing.Icon? _iconE;
        private int _devTapCount = 0;
        private DateTime _lastDevTap = DateTime.MinValue;

        [DllImport("user32.dll", SetLastError = true)]
        static extern bool DestroyIcon(IntPtr hIcon);

        public MainWindow(MainViewModel viewModel)
        {
            InitializeComponent();
            _viewModel = viewModel;
            DataContext = _viewModel;

            InitializeTrayIcon();
            
            _viewModel.PropertyChanged += (s, e) =>
            {
                if (e.PropertyName == nameof(MainViewModel.IsVietnameseMode))
                {
                    UpdateTrayIcon(_viewModel.IsVietnameseMode);
                }
            };

            // Register global hotkey callback
            App.EngineService.OnOpenWindowRequested += () => Dispatcher.Invoke(ToggleWindowVisibility);
        }

        private void ToggleWindowVisibility()
        {
            if (IsVisible)
            {
                if (WindowState == WindowState.Minimized)
                {
                    ShowWindow();
                }
                else
                {
                    Hide();
                }
            }
            else
            {
                ShowWindow();
            }
        }

        private void TitleBar_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            if (e.LeftButton == MouseButtonState.Pressed)
            {
                DragMove();
            }
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Hide();
        }

        private void ExitButton_Click(object sender, RoutedEventArgs e)
        {
            Cleanup();
            System.Windows.Application.Current.Shutdown();
        }

        private void InitializeTrayIcon()
        {
            _notifyIcon = new System.Windows.Forms.NotifyIcon();
            LoadIcons();
            UpdateTrayIcon(_viewModel.IsVietnameseMode);

            _notifyIcon.Visible = true;
            _notifyIcon.Text = "VNKey 2.0 (MVVM)";
            _notifyIcon.DoubleClick += (s, e) => ShowWindow();
            _notifyIcon.MouseClick += (s, e) =>
            {
                if (e.Button == System.Windows.Forms.MouseButtons.Left)
                {
                    _viewModel.IsVietnameseMode = !_viewModel.IsVietnameseMode;
                }
            };

            var contextMenu = new System.Windows.Forms.ContextMenuStrip();
            contextMenu.Items.Add("Hiện bảng điều khiển", null, (s, e) => ShowWindow());
            contextMenu.Items.Add("Thoát", null, (s, e) => ExitButton_Click(s, null!));
            _notifyIcon.ContextMenuStrip = contextMenu;
        }

        private void LoadIcons()
        {
            _iconV = LoadIconFromAssets("V.png");
            _iconE = LoadIconFromAssets("E.png");
        }

        private System.Drawing.Icon? LoadIconFromAssets(string name)
        {
            try
            {
                var iconUri = new Uri($"pack://application:,,,/Assets/{name}");
                var streamInfo = System.Windows.Application.GetResourceStream(iconUri);
                if (streamInfo != null)
                {
                    using var stream = streamInfo.Stream;
                    using var bmp = new System.Drawing.Bitmap(stream);
                    IntPtr hIcon = bmp.GetHicon();
                    System.Drawing.Icon icon = System.Drawing.Icon.FromHandle(hIcon);
                    return icon;
                }
            }
            catch { }
            return null;
        }

        private void UpdateTrayIcon(bool isVietnamese)
        {
            if (_notifyIcon == null) return;
            var targetIcon = isVietnamese ? _iconV : _iconE;
            if (targetIcon != null)
            {
                _notifyIcon.Icon = targetIcon;
            }
        }

        private void ShowWindow()
        {
            Show();
            WindowState = WindowState.Normal;
            Activate();
        }

        private void SearchToggle_Checked(object sender, RoutedEventArgs e)
        {
            // Focus the search box when it expands
            SearchTextBox.Focus();
        }

        private void SearchToggle_Unchecked(object sender, RoutedEventArgs e)
        {
            // Clear the search text when it collapses
            if (DataContext is ViewModels.MainViewModel vm)
            {
                vm.ShorthandSearchText = string.Empty;
            }
        }

        private void Window_PreviewMouseDown(object sender, MouseButtonEventArgs e)
        {
            // Auto close search box if clicking outside of it
            if (SearchToggle.IsChecked == true)
            {
                var point = e.GetPosition(SearchBorder);
                if (point.X < 0 || point.Y < 0 || point.X > SearchBorder.ActualWidth || point.Y > SearchBorder.ActualHeight)
                {
                    SearchToggle.IsChecked = false;
                    // Ensure focus is restored to the window or previous element so typing works
                    Keyboard.ClearFocus(); 
                }
            }
        }

        private void Cleanup()
        {
            _notifyIcon?.Dispose();
            if (_iconV != null) { DestroyIcon(_iconV.Handle); _iconV.Dispose(); }
            if (_iconE != null) { DestroyIcon(_iconE.Handle); _iconE.Dispose(); }
        }

        protected override void OnClosing(System.ComponentModel.CancelEventArgs e)
        {
            // Instead of closing, we just hide to stay in tray
            e.Cancel = true;
            Hide();
            base.OnClosing(e);
        }

        private void VersionText_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            var now = DateTime.Now;
            if ((now - _lastDevTap).TotalSeconds > 2)
                _devTapCount = 0;

            _lastDevTap = now;
            _devTapCount++;

            if (_devTapCount >= 5)
            {
                _devTapCount = 0;
                _viewModel.IsDevModeEnabled = !_viewModel.IsDevModeEnabled;
                if (_viewModel.IsDevModeEnabled)
                {
                    _viewModel.CurrentPage = "Diagnostics";
                    System.Windows.MessageBox.Show("🔧 Dev Mode đã được kích hoạt!", "Developer Mode", MessageBoxButton.OK, MessageBoxImage.Information);
                }
                else
                {
                    _viewModel.CurrentPage = "Input";
                    System.Windows.MessageBox.Show("Dev Mode đã được tắt.", "Developer Mode", MessageBoxButton.OK, MessageBoxImage.Information);
                }
            }
        }

        protected override void OnPreviewKeyDown(System.Windows.Input.KeyEventArgs e)
        {
            if (_viewModel.IsRecordingShortcut)
            {
                var key = e.Key == Key.System ? e.SystemKey : e.Key;

                // Ignore pure modifier presses
                if (key != Key.LeftCtrl && key != Key.RightCtrl &&
                    key != Key.LeftAlt && key != Key.RightAlt &&
                    key != Key.LeftShift && key != Key.RightShift &&
                    key != Key.LWin && key != Key.RWin)
                {
                    var parts = new System.Collections.Generic.List<string>();
                    if (Keyboard.Modifiers.HasFlag(ModifierKeys.Control)) parts.Add("Ctrl");
                    if (Keyboard.Modifiers.HasFlag(ModifierKeys.Alt)) parts.Add("Alt");
                    if (Keyboard.Modifiers.HasFlag(ModifierKeys.Shift)) parts.Add("Shift");
                    if (Keyboard.Modifiers.HasFlag(ModifierKeys.Windows)) parts.Add("Win");

                    string keyName = key.ToString();
                    // Map to expected format
                    if (key >= Key.D0 && key <= Key.D9) keyName = "D" + (key - Key.D0);
                    
                    parts.Add(keyName);

                    _viewModel.CustomShortcut = string.Join("+", parts);
                    _viewModel.IsRecordingShortcut = false;
                    e.Handled = true;
                    return;
                }
            }
            base.OnPreviewKeyDown(e);
        }

        private void TemplateButton_Click(object sender, RoutedEventArgs e)
        {
            if (sender is FrameworkElement element)
            {
                if (element.ContextMenu != null)
                {
                    element.ContextMenu.PlacementTarget = element;
                    element.ContextMenu.IsOpen = true;
                }
            }
        }
    }
}

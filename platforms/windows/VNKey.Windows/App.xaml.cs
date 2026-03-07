using System;
using System.Windows;

namespace VNKey.Windows
{
    public partial class App : System.Windows.Application
    {
        private static System.Threading.Mutex? _mutex;

        public static Models.AppConfig Config { get; set; } = Models.AppConfig.Load();

        // Services
        public static Services.IEngineService EngineService { get; private set; } = null!;
        public static Services.IConfigService ConfigService { get; private set; } = null!;
        public static Services.IThemeService ThemeService { get; private set; } = null!;

        // ViewModels
        public static ViewModels.MainViewModel MainViewModel { get; private set; } = null!;
        public static ViewModels.DiagnosticsViewModel DiagnosticsViewModel { get; private set; } = null!;

        protected override void OnStartup(StartupEventArgs e)
        {
            const string appName = "VNKey.Windows.App";
            _mutex = new System.Threading.Mutex(true, appName, out bool createdNew);

            if (!createdNew)
            {
                // Already running
                System.Windows.MessageBox.Show("VNKey đã đang chạy.", "Thông báo", System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Information);
                System.Windows.Application.Current.Shutdown();
                return;
            }

            // Initialize Services
            ConfigService = new Services.ConfigService();
            EngineService = new Services.EngineService();
            ThemeService = new Services.ThemeService();

            // Initialize ViewModels
            DiagnosticsViewModel = new ViewModels.DiagnosticsViewModel(EngineService);
            MainViewModel = new ViewModels.MainViewModel(EngineService, ConfigService, ThemeService, DiagnosticsViewModel);

            Config = ConfigService.Config;
            
            // Start Engine Hook
            EngineService.StartHook();

            // Show MainWindow
            var mainWindow = new Views.MainWindow(MainViewModel);
            mainWindow.Show();

            base.OnStartup(e);
        }

        protected override void OnExit(System.Windows.ExitEventArgs e)
        {
            (EngineService as IDisposable)?.Dispose();
            _mutex?.ReleaseMutex();
            base.OnExit(e);
        }
    }
}

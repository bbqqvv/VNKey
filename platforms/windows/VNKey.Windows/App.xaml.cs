using System;
using System.Windows;

namespace VNKey.Windows
{
    public partial class App : System.Windows.Application
    {
        private static System.Threading.Mutex? _mutex;

        public static Core.AppConfig Config { get; set; } = Core.AppConfig.Load();

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

            Config = Core.AppConfig.Load();
            base.OnStartup(e);
        }

        protected override void OnExit(System.Windows.ExitEventArgs e)
        {
            _mutex?.ReleaseMutex();
            base.OnExit(e);
        }
    }
}

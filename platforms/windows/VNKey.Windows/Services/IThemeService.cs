namespace VNKey.Windows.Services
{
    public interface IThemeService
    {
        bool IsDarkMode { get; set; }
        void ApplyTheme(bool isDarkMode);
        bool GetWindowsTheme();
    }
}

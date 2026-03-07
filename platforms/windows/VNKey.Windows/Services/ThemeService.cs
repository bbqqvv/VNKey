using System;
using Microsoft.Win32;
using System.Windows.Media;

namespace VNKey.Windows.Services
{
    public class ThemeService : IThemeService
    {
        private bool? _isDarkMode = null;

        public bool IsDarkMode
        {
            get => _isDarkMode ?? false;
            set
            {
                if (_isDarkMode != value)
                {
                    _isDarkMode = value;
                    ApplyTheme(value);
                }
            }
        }

        public void ApplyTheme(bool isDarkMode)
        {
            try
            {
                var app = System.Windows.Application.Current;
                if (app == null) return;

                if (isDarkMode)
                {
                    // Dark Mode - Fluent Dark Palette
                    SetResource(app, "AppBackground", "#1E1E1E");
                    SetResource(app, "AppSidebarBackground", "#252526");
                    SetResource(app, "AppControlBackground", "#2D2D30");
                    SetResource(app, "AppCardBackground", "#252526");
                    SetResource(app, "AppAccent", "#00B7C3");
                    SetResource(app, "AppAccentForeground", "#FFFFFF");
                    SetResource(app, "AppAccentHoverBackground", "#00D1E0");
                    SetResource(app, "AppBorder", "#3F3F46");
                    SetResource(app, "AppText", "#FFFFFF");
                    SetResource(app, "AppSecondaryText", "#D1D1D1");
                    SetResource(app, "AppButtonBackground", "#3E3E42");
                    SetResource(app, "AppButtonHoverBackground", "#4E4E52");
                    SetResource(app, "AppInputBackground", "#1E1E1E");
                    SetResource(app, "AppInputForeground", "#FFFFFF");
                    SetResource(app, "AppAlternatingRow", "#15FFFFFF");
                    SetResource(app, "AppScrollbarThumb", "#4E4E52");
                }
                else
                {
                    // Light Mode - Fluent Light Palette
                    SetResource(app, "AppBackground", "#F3F3F3");
                    SetResource(app, "AppSidebarBackground", "#EAEAEA");
                    SetResource(app, "AppControlBackground", "#FFFFFF");
                    SetResource(app, "AppCardBackground", "#FFFFFF");
                    SetResource(app, "AppAccent", "#0078D7");
                    SetResource(app, "AppAccentForeground", "#FFFFFF");
                    SetResource(app, "AppAccentHoverBackground", "#106EBE");
                    SetResource(app, "AppBorder", "#D1D1D1");
                    SetResource(app, "AppText", "#000000");
                    SetResource(app, "AppSecondaryText", "#5D5D5D");
                    SetResource(app, "AppButtonBackground", "#E1E1E1");
                    SetResource(app, "AppButtonHoverBackground", "#CCCCCC");
                    SetResource(app, "AppInputBackground", "#FFFFFF");
                    SetResource(app, "AppInputForeground", "#000000");
                    SetResource(app, "AppAlternatingRow", "#0A000000");
                    SetResource(app, "AppScrollbarThumb", "#CCCCCC");
                }
            }
            catch { }
        }

        private void SetResource(System.Windows.Application app, string key, string hex)
        {
            try
            {
                System.Windows.Media.Color color;
                if (hex.Length == 9) // ARGB hex
                {
                    color = (System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString(hex);
                }
                else
                {
                    color = (System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString(hex);
                }
                app.Resources[key] = new System.Windows.Media.SolidColorBrush(color);
            }
            catch 
            {
                // Fallback for alpha hex if Converter fails
                if (hex.StartsWith("#") && hex.Length == 9)
                {
                     byte a = Convert.ToByte(hex.Substring(1, 2), 16);
                     byte r = Convert.ToByte(hex.Substring(3, 2), 16);
                     byte g = Convert.ToByte(hex.Substring(5, 2), 16);
                     byte b = Convert.ToByte(hex.Substring(7, 2), 16);
                     app.Resources[key] = new System.Windows.Media.SolidColorBrush(System.Windows.Media.Color.FromArgb(a, r, g, b));
                }
            }
        }

        public bool GetWindowsTheme()
        {
            try
            {
                using (var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"))
                {
                    if (key != null)
                    {
                        object registryValue = key.GetValue("AppsUseLightTheme");
                        if (registryValue is int val) return val == 0;
                    }
                }
            }
            catch { }
            return false;
        }
    }
}

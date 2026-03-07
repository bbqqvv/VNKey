using System;
using System.Globalization;
using System.Windows.Data;
using VNKey.Windows.Models;

namespace VNKey.Windows.Converters
{
    public class InputModeToIndexConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is InputMode mode)
            {
                return (int)mode;
            }
            return 0;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is int index)
            {
                return (InputMode)index;
            }
            return InputMode.Telex;
        }
    }
}

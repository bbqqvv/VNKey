using System;
using System.Globalization;
using System.Windows.Data;

namespace VNKey.Windows.Converters
{
    public class EnumToBooleanConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value == null || parameter == null) return false;
            return value.ToString().Equals(parameter.ToString(), StringComparison.InvariantCultureIgnoreCase);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value == null || parameter == null) return System.Windows.Data.Binding.DoNothing;
            if ((bool)value)
            {
                if (targetType.IsEnum)
                {
                    return Enum.Parse(targetType, parameter.ToString());
                }
                // If target is string, just return the parameter string
                if (targetType == typeof(string))
                {
                    return parameter.ToString();
                }
            }
            return System.Windows.Data.Binding.DoNothing;
        }
    }
}

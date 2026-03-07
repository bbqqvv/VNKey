using System;
using System.Windows;
using System.Windows.Controls;
using VNKey.Windows.ViewModels;

namespace VNKey.Windows.Controls
{
    public partial class DiagnosticsControl : System.Windows.Controls.UserControl
    {
        public event Action? PopOutRequested;

        public DiagnosticsControl()
        {
            InitializeComponent();
        }

        private void BtnPopOut_Click(object sender, RoutedEventArgs e)
        {
            PopOutRequested?.Invoke();
        }
    }
}

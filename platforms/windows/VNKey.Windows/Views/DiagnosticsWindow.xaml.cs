using System;
using System.Windows;
using System.Windows.Input;
using VNKey.Windows.ViewModels;

namespace VNKey.Windows.Views
{
    public partial class DiagnosticsWindow : Window
    {
        public DiagnosticsWindow(DiagnosticsViewModel viewModel)
        {
            InitializeComponent();
            DataContext = viewModel;
        }

        private void TitleBar_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            DragMove();
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}

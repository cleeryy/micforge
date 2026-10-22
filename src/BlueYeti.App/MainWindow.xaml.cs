using System.Windows;
using BlueYeti.Core;

namespace BlueYeti.App;

public partial class MainWindow : Window
{
    private readonly AudioEngine _engine = new();

    public MainWindow()
    {
        InitializeComponent();
        Loaded += (_, _) => RefreshDevices();
        _engine.PeakChanged += peak =>
            Dispatcher.Invoke(() => PeakMeter.Value = Math.Clamp(peak * 100, 0, 100));
    }

    private void RefreshDevices()
    {
        // TODO: enumerate MMDeviceEnumerator (NAudio) -> InputSelector / OutputSelector,
        // preselect Blue Yeti as input and CABLE Input as output.
        InputSelector.Items.Clear();
        OutputSelector.Items.Clear();
    }

    private void OnRescan(object sender, RoutedEventArgs e) => RefreshDevices();

    private void OnStartStop(object sender, RoutedEventArgs e)
    {
        // TODO: Start(input, output) / Stop() + update the button label.
        MessageBox.Show("TODO: wire up AudioEngine + NAudio.", "Yeti Voice");
    }

    protected override void OnClosed(EventArgs e)
    {
        _engine.Dispose();
        base.OnClosed(e);
    }
}

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
        // TODO: énumérer MMDeviceEnumerator (NAudio) -> InputSelector / OutputSelector,
        // présélectionner Blue Yeti en entrée et CABLE Input en sortie.
        InputSelector.Items.Clear();
        OutputSelector.Items.Clear();
    }

    private void OnRescan(object sender, RoutedEventArgs e) => RefreshDevices();

    private void OnStartStop(object sender, RoutedEventArgs e)
    {
        // TODO: Start(input, output) / Stop() + MAJ du libellé du bouton.
        MessageBox.Show("TODO: câbler AudioEngine + NAudio.", "Yeti Voice");
    }

    protected override void OnClosed(EventArgs e)
    {
        _engine.Dispose();
        base.OnClosed(e);
    }
}

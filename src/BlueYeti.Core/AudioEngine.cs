using BlueYeti.Core.Models;

namespace BlueYeti.Core;

/// <summary>
/// Moteur audio (stub) : capture WASAPI de l'entrée, traverse la <see cref="DspChain"/>,
/// restitue vers la sortie (câble virtuel). TODO: câbler NAudio (WasapiCapture /
/// WasapiOut) côté App, le Core restant testable sans device.
/// </summary>
public sealed class AudioEngine : IDisposable
{
    private readonly DspChain _chain = new();
    private bool _disposed;

    public bool IsRunning { get; private set; }
    public float LastPeak { get; private set; }

    public event Action<float>? PeakChanged;

    public void LoadPreset(Preset preset) => _chain.LoadPreset(preset);

    public void Start(string inputDeviceId, string outputDeviceId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(inputDeviceId);
        ArgumentException.ThrowIfNullOrWhiteSpace(outputDeviceId);
        // TODO: ouvrir WasapiCapture(input) + WasapiOut(output), boucle Process().
        IsRunning = true;
    }

    public void Stop()
    {
        // TODO: arrêter et disposer les devices NAudio.
        IsRunning = false;
        LastPeak = 0f;
        PeakChanged?.Invoke(0f);
    }

    public void Dispose()
    {
        if (_disposed) return;
        if (IsRunning) Stop();
        _disposed = true;
    }
}

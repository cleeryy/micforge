using BlueYeti.Core.Models;

namespace BlueYeti.Core;

/// <summary>Stub de la chaîne DSP : applique les étages dans l'ordre,
/// pass-through pour l'instant (TODO: implémenter chaque étage).</summary>
public sealed class DspChain
{
    public Preset Preset { get; private set; } = new();

    public void LoadPreset(Preset preset) =>
        Preset = preset ?? throw new ArgumentNullException(nameof(preset));

    /// <summary>Traite un buffer entrelacé 48 kHz / float32. Retourne le pic (0..1).</summary>
    public float Process(Span<float> buffer)
    {
        // TODO: high-pass -> rnnoise -> gate -> comp -> eq -> deesser -> limiter
        float peak = 0f;
        foreach (var s in buffer)
        {
            var a = Math.Abs(s);
            if (a > peak) peak = a;
        }
        return peak;
    }
}

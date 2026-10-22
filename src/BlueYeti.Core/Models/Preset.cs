using System.Text.Json;
using System.Text.Json.Serialization;

namespace BlueYeti.Core.Models;

public sealed class HighpassSettings { public bool Enabled { get; set; } = true; public int FrequencyHz { get; set; } = 80; }
public sealed class RnnoiseSettings { public bool Enabled { get; set; } = true; }
public sealed class GateSettings { public bool Enabled { get; set; } = true; public float ThresholdDb { get; set; } = -40f; public float AttackMs { get; set; } = 5f; public float ReleaseMs { get; set; } = 120f; }
public sealed class CompSettings { public bool Enabled { get; set; } = true; public float ThresholdDb { get; set; } = -18f; public float Ratio { get; set; } = 3f; public float AttackMs { get; set; } = 10f; public float ReleaseMs { get; set; } = 150f; public float MakeupDb { get; set; } = 4f; }
public sealed class EqBand { public float FrequencyHz { get; set; }; public float GainDb { get; set; }; public float Q { get; set; } = 1f; }
public sealed class EqSettings { public bool Enabled { get; set; } = true; public List<EqBand> Bands { get; set; } = new(); }
public sealed class DeesserSettings { public bool Enabled { get; set; } = true; public float FrequencyHz { get; set; } = 6500f; public float ThresholdDb { get; set; } = -24f; }
public sealed class LimiterSettings { public bool Enabled { get; set; } = true; public float CeilingDb { get; set; } = -1f; public float ReleaseMs { get; set; } = 50f; }

public sealed class Preset
{
    public int Version { get; set; } = 1;
    public string Name { get; set; } = string.Empty;
    public HighpassSettings Highpass { get; set; } = new();
    public RnnoiseSettings Rnnoise { get; set; } = new();
    public GateSettings Gate { get; set; } = new();
    public CompSettings Comp { get; set; } = new();
    public EqSettings Eq { get; set; } = new();
    public DeesserSettings Deesser { get; set; } = new();
    public LimiterSettings Limiter { get; set; } = new();
    public string Notes { get; set; } = string.Empty;

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.Never,
    };

    public string ToJson() => JsonSerializer.Serialize(this, JsonOptions);
    public static Preset FromJson(string json) =>
        JsonSerializer.Deserialize<Preset>(json, JsonOptions)
        ?? throw new InvalidOperationException("Preset JSON invalide.");
    public static Preset LoadFromFile(string path) => FromJson(File.ReadAllText(path));
    public void SaveToFile(string path) => File.WriteAllText(path, ToJson());
}

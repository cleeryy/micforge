using BlueYeti.Core.Models;

namespace BlueYeti.Core.Tests;

public sealed class PresetTests
{
    [Fact]
    public void Preset_Serialization_RoundTrip()
    {
        var preset = new Preset
        {
            Name = "test",
            Notes = "Allo",
            Eq = new EqSettings
            {
                Bands = new() { new EqBand { FrequencyHz = 1000f, GainDb = -2f, Q = 1f } }
            }
        };

        var json = preset.ToJson();
        var back = Preset.FromJson(json);

        Assert.Equal(1, back.Version);
        Assert.Equal("test", back.Name);
        Assert.Equal("Allo", back.Notes);
        Assert.Single(back.Eq.Bands);
        Assert.Equal(1000f, back.Eq.Bands[0].FrequencyHz);
    }
}

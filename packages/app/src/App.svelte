<script lang="ts">
import { RadioButtonGroup, RadioButton } from "carbon-components-svelte";
import {Theme, theme} from "./utils/theme";

</script>

<h1>Nix Navigator</h1>

<RadioButtonGroup
    orientation="vertical"
    legendText="Theme"
    name="theme"
    bind:selected={$theme}
>
    <RadioButton labelText="System" value="{Theme.System}" />
    <RadioButton labelText="Dark" value="{Theme.Dark}" />
    <RadioButton labelText="Light" value="{Theme.Light}" />
</RadioButtonGroup>
<script lang="ts">
import { Button, Toolbar, ToolbarContent, ContentSwitcher, Switch } from "carbon-components-svelte";
import { View, ViewFilled, IntrusionPrevention, Settings } from "carbon-icons-svelte";
import { theme } from "./utils/theme";

const margin = 5;
const style = `margin-left:${margin}px;margin-right:${margin}px;`;
</script>

<Toolbar size="sm">
    <ToolbarContent>
        <ContentSwitcher lowContrast size="sm" bind:selectedIndex={$theme} {style}>
            <Switch
                icon={IntrusionPrevention}
                text="Theme: System"
                tooltipAlignment="end"
            />
            <Switch
                icon={View}
                text="Theme: Light"
                tooltipAlignment="end"
            />
            <Switch
                icon={ViewFilled}
                text="Theme: Dark"
                tooltipAlignment="end"
            />
        </ContentSwitcher>
        <Button
            kind="tertiary"
            size="small"
            icon={Settings}
            tooltipAlignment="end"
            iconDescription="Configuration"
            class="toolbar-element"
            {style}
        />
    </ToolbarContent>
</Toolbar>

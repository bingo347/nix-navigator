import { invoke } from "@tauri-apps/api/core";
import { writable, readable } from "svelte/store";

export const enum Theme {
    System = 0,
    Light = 1,
    Dark = 2,
}

export const theme = writable(Theme.System);

invoke<Theme>("themeGet").then((value) => {
    theme.set(value);
    const themeInternal = readable(value, (set) => {
        const themeMedia = window.matchMedia("(prefers-color-scheme: dark)");
        const matchesToTheme = ({ matches }: { matches: boolean }) => matches ? Theme.Dark : Theme.Light;

        let current = value;
        let systemTheme = matchesToTheme(themeMedia);

        const update = () => {
            invoke<void>("themeSet", { theme: current });
            if (current === Theme.System) {
                set(systemTheme);
            } else {
                set(current);
            }
        };

        const unsubscribe = theme.subscribe((value) => {
            if (value === current) {
                return;
            }
            current = value;
            update();
        });

        const systemThemeListener = (event: MediaQueryListEvent) => {
            systemTheme = matchesToTheme(event);
            update();
        };

        themeMedia.addEventListener("change", systemThemeListener);

        update();

        return () => {
            unsubscribe();
            themeMedia.removeEventListener("change", systemThemeListener);
        };
    });

    themeInternal.subscribe((value) => {
        const { documentElement } = document;
        switch (value) {
            case Theme.Dark:
                documentElement.setAttribute("theme", "g90");
                break;
            case Theme.Light:
                documentElement.setAttribute("theme", "g10");
                break;
        }
    });
});




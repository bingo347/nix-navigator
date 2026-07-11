import { writable, readable } from "svelte/store";

export const enum Theme {
    System,
    Light,
    Dark,
}

export const theme = writable(Theme.System);

const themeInternal = readable(Theme.System, (set) => {
    const themeMedia = window.matchMedia("(prefers-color-scheme: dark)");
    const matchesToTheme = ({ matches }: { matches: boolean }) => matches ? Theme.Dark : Theme.Light;

    let current = Theme.System;
    let systemTheme = matchesToTheme(themeMedia);

    const update = () => {
        if (current === Theme.System) {
            set(systemTheme);
        } else {
            set(current);
        }
    };

    const unsubscribe = theme.subscribe((value) => {
        current = value;
        update();
    });

    const systemThemeListener = (event: MediaQueryListEvent) => {
        systemTheme = matchesToTheme(event);
        update();
    };

    themeMedia.addEventListener("change", systemThemeListener);

    return () => {
        unsubscribe();
        themeMedia.removeEventListener("change", systemThemeListener);
    };
});

themeInternal.subscribe((value) => {
    const { classList } = document.body;
    switch (value) {
        case Theme.Dark:
            classList.remove("light");
            classList.add("dark");
            break;
        case Theme.Light:
            classList.remove("dark");
            classList.add("light");
            break;
    }
});

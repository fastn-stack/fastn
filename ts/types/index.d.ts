export {};

declare global {
    interface Window {
        ftd: any;
        enable_dark_mode(): void;
        enable_light_mode(): void;
        enable_system_mode(): void;
    }

    interface Export {
        init: object;
        handle_event(evt: Event, id: string, event: string, obj: Element): void;
    }
}
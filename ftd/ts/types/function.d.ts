export {};

declare global {
    interface ActionValue {
        [key: string]: FunctionArgument | string;
    }

    interface Action {
        name: string;
        values: ActionValue;
    }

    interface FunctionArgument {
        value: any;
        reference: string | null;
        mutable: boolean;
    }
}
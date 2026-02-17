// Modal Form Controller
import { GAME_STRATEGY, SERVERS } from "./constants.js";

export class FormController {
    constructor(formSelector = "form#account") {
        this.form = document.$(formSelector);
    }

    getData() {
        const data = this.form.value;
        data.strategy = GAME_STRATEGY;
        return data;
    }

    setData(data) {
        const { username, password, server, nickname } = data;
        this.form.value = { 
            username, 
            password, 
            strategy: GAME_STRATEGY, 
            server, 
            nickname 
        };
    }

    forceStrategy() {
        const strategyElement = document.$("[name='strategy']");
        if (strategyElement) {
            strategyElement.value = GAME_STRATEGY;
        }
    }

    onSubmit(callback) {
        document.on("click", "button#submit", () => {
            callback(this.getData());
        });
    }
}

export const initializeForm = (isEditMode = false) => {
    const controller = new FormController();
    
    controller.onSubmit((data) => {
        Window.this.close(data);
    });

    document.on("ready", () => {
        controller.forceStrategy();
        
        if (isEditMode) {
            const { account } = Window.this.parameters;
            if (account) {
                controller.setData(account);
            }
        }
    });
};

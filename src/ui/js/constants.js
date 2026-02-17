// Game Configuration
export const GAME_STRATEGY = "337.lua";

// Server Configuration
export const SERVERS = [
    { id: "10000", name: "Ilha dos valentões", range: "S1-3, 9-10, 12-19" },
    { id: "10001", name: "Vale dos Ouriços", range: "S4-8, 11, 20-46" },
    { id: "10005", name: "Jogos Olímpicos", range: "S47-131, 362-375" },
    { id: "10006", name: "Lugares escuros", range: "S132-394" },
    { id: "10031", name: "Universo DDToker", range: "S395-398" },
    { id: "10090", name: "Legado dos Campeões", range: "S399" },
    { id: "10091", name: "Aurora", range: "S400" }
];

export const SERVER_NAMES = SERVERS.reduce((acc, server) => {
    acc[server.id] = server.name;
    return acc;
}, {});

// UI Configuration
export const MODAL_CONFIG = {
    width: "28em",
    height: "30em"
};

// Form Field Configuration
export const FORM_FIELDS = {
    username: {
        type: "text",
        label: "Usuário",
        placeholder: "Digite seu usuário",
        required: true
    },
    password: {
        type: "password",
        label: "Senha",
        placeholder: "Digite sua senha",
        required: true
    },
    server: {
        type: "select",
        label: "Servidor",
        required: true
    },
    nickname: {
        type: "text",
        label: "Apelido (opcional)",
        placeholder: "Como deseja identificar esta conta",
        required: false
    }
};

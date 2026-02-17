import * as account_db from "./accountdb.js";
import { SERVER_NAMES, SERVERS, GAME_STRATEGY } from "./constants.js";
import { 
    getAccountDisplayName, 
    filterAccounts, 
    sortByLastUsed, 
    isValidGameUrl 
} from "./utils.js";

const { signal } = Reactor;

// State Management
const state = {
    accounts: signal(account_db.get_all_accounts()),
    loadingAccounts: signal({}),
    searchText: signal("")
};

// Remover conta de teste apos inicializar
const TEST_ACCOUNT_UUID = "00000000-0000-0000-0000-000000000001";
if (state.accounts.value[TEST_ACCOUNT_UUID]) {
    account_db.delete_account(TEST_ACCOUNT_UUID);
    state.accounts.value = Object.assign({}, account_db.get_all_accounts());
}

// Computed Values
const filteredAccounts = () => {
    const filtered = filterAccounts(state.accounts.value, state.searchText.value);
    const sorted = sortByLastUsed(filtered);
    return Object.fromEntries(sorted);
};

// UI Components
const Header = () => (
    <header>
        <h1 style="color: #ffffff; font-size: 2.6em; margin: 0; font-weight: 700; letter-spacing: -0.3px; text-shadow: 0 2px 20px rgba(102, 126, 234, 0.5);">
            DDTank Launcher
        </h1>
        <p style="color: rgba(255, 255, 255, 0.7); margin-top: 12px; font-size: 1.05em; font-weight: 500; letter-spacing: 0.3px;">
            Gerenciador de Contas de Batalha
        </p>
        <SearchBar />
        <ActionButtons />
    </header>
);

const SearchBar = () => (
    <div style="margin-top: 20px;">
        <input 
            type="text" 
            placeholder="🔍 Buscar conta..." 
            class="search-input"
            oninput={(e) => state.searchText.value = e.target.value}
            value={state.searchText.value} 
        />
    </div>
);

const ActionButtons = () => (
    <div style="margin-top: 15px;">
        <button onclick={AccountActions.showAddDialog}>➕ Adicionar Conta</button>
        <button onclick={AccountActions.refresh}>🔄 Atualizar Lista</button>
        <button onclick={() => Window.this.xcall('open_reguinha')}>📏 Abrir Régua</button>
    </div>
);

const AccountCard = ({ accountId, account }) => {
    const isLoading = state.loadingAccounts.value[accountId];
    const server = SERVERS.find(s => s.id === account.server);
    const serverDisplay = server ? `${server.name} (${server.range})` : account.server;
    const displayName = getAccountDisplayName(account);
    
    return (
        <div 
            class={isLoading ? "account loading" : "account"}
            onclick={() => !isLoading && AccountActions.login(accountId)}
        >
            <div class="account-content">
                <div class="account-avator">
                    {isLoading ? <div class="card-spinner"></div> : <img src="img/logo.png" />}
                </div>
                <div class="account-detail">{displayName}</div>
            </div>
            <div class="server-badge">🌐 {serverDisplay}</div>
            <div class="account-actions">
                <button class="action-btn" onclick={() => { AccountActions.showEditDialog(accountId); return false; }}>✏️</button>
                <button class="action-btn delete" onclick={() => { AccountActions.delete(accountId); return false; }}>🗑️</button>
            </div>
        </div>
    );
};

const AccountList = () => (
    <main>
        <div id="account-list">
            {Object.entries(filteredAccounts()).map(([accountId, account]) => 
                <AccountCard accountId={accountId} account={account} />
            )}
        </div>
    </main>
);

// Business Logic
const AccountActions = {
    login: (accountId) => {
        if (state.loadingAccounts.value[accountId]) return;

        const account = account_db.get_account(accountId);
        const { strategy, username, password, server } = account;

        // Update last used timestamp
        account.last_used = Date.now();
        account_db.replace_account(accountId, account);
        // Forcar atualizacao do estado criando nova referencia
        state.accounts.value = Object.assign({}, account_db.get_all_accounts());

        // Set loading state
        state.loadingAccounts.value = { ...state.loadingAccounts.value, [accountId]: true };

        Window.this.xcall("login", strategy, username, password, server, (response) => {
            // Clear loading state
            const newLoading = { ...state.loadingAccounts.value };
            delete newLoading[accountId];
            state.loadingAccounts.value = newLoading;

            // Atualizar lista de contas para refletir nova ordenação por last_used
            state.accounts.value = Object.assign({}, account_db.get_all_accounts());

            if (isValidGameUrl(response)) {
                Window.this.xcall("play_flash", response);
            } else if (!response.startsWith("Abrindo jogo")) {
                Window.this.modal(<error>❌ Erro ao conectar:<br/><br/>{response}</error>);
            }
        });
    },

    showAddDialog: () => {
        const data = Window.this.modal({
            url: __DIR__ + "../htm/add-account.htm",
            parameters: { strategy_list: Window.this.xcall("get_all_strategy") }
        });

        if (data) {
            const { username, password, strategy, server, nickname } = data;
            const success = account_db.add_account(username, password, strategy, server, nickname);
            if (success) {
                // Forcar atualizacao do estado criando nova referencia
                state.accounts.value = Object.assign({}, account_db.get_all_accounts());
            } else {
                Window.this.modal(<error>❌ Erro ao adicionar conta</error>);
            }
        }
    },

    showEditDialog: (accountId) => {
        const account = account_db.get_account(accountId);
        const data = Window.this.modal({
            url: __DIR__ + "../htm/edit-account.htm",
            parameters: {
                strategy_list: Window.this.xcall("get_all_strategy"),
                account
            }
        });

        if (data) {
            const { username, password, strategy, server, nickname } = data;
            Object.assign(account, { username, password, strategy, server, nickname });
            const success = account_db.replace_account(accountId, account);
            if (success) {
                // Forcar atualizacao do estado criando nova referencia
                state.accounts.value = Object.assign({}, account_db.get_all_accounts());
            } else {
                Window.this.modal(<error>❌ Erro ao atualizar conta</error>);
            }
        }
    },

    delete: (accountId) => {
        const success = account_db.delete_account(accountId);
        if (success) {
            // Forcar atualizacao do estado criando nova referencia
            state.accounts.value = Object.assign({}, account_db.get_all_accounts());
        } else {
            Window.this.modal(<error>❌ Erro ao deletar conta</error>);
        }
    },

    refresh: () => {
        // Forcar atualizacao do estado criando nova referencia
        state.accounts.value = Object.assign({}, account_db.get_all_accounts());
    }
};

// Main App Component
export const App = () => (
    <div>
        <Header />
        <AccountList />
    </div>
);

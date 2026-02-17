/**
 * Utility functions for account management
 */

/**
 * Get display name for an account
 * @param {Object} account - Account object
 * @returns {string} Display name
 */
export const getAccountDisplayName = (account) => {
    return (account.nickname && account.nickname.trim()) || account.username;
};

/**
 * Filter accounts by search text
 * @param {Object} accounts - Accounts object
 * @param {string} searchText - Search text
 * @returns {Array} Filtered accounts array
 */
export const filterAccounts = (accounts, searchText) => {
    const search = searchText.toLowerCase();
    let accountsArray = Object.entries(accounts);
    
    if (search) {
        accountsArray = accountsArray.filter(([id, account]) => {
            const name = getAccountDisplayName(account).toLowerCase();
            return name.includes(search);
        });
    }
    
    return accountsArray;
};

/**
 * Sort accounts by last used timestamp
 * @param {Array} accountsArray - Array of [id, account] tuples
 * @returns {Array} Sorted accounts array
 */
export const sortByLastUsed = (accountsArray) => {
    return accountsArray.sort((a, b) => {
        const lastUsedA = a[1].last_used || 0;
        const lastUsedB = b[1].last_used || 0;
        return lastUsedB - lastUsedA;
    });
};

/**
 * Validate form data
 * @param {Object} data - Form data
 * @param {Array} requiredFields - Array of required field names
 * @returns {Object} { valid: boolean, errors: Array }
 */
export const validateFormData = (data, requiredFields = ['username', 'password', 'server']) => {
    const errors = [];
    
    requiredFields.forEach(field => {
        if (!data[field] || !data[field].trim()) {
            errors.push(`${field} é obrigatório`);
        }
    });
    
    return {
        valid: errors.length === 0,
        errors
    };
};

/**
 * Debounce function
 * @param {Function} func - Function to debounce
 * @param {number} wait - Wait time in ms
 * @returns {Function} Debounced function
 */
export const debounce = (func, wait = 300) => {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
};

/**
 * Check if URL is valid game URL
 * @param {string} url - URL to check
 * @returns {boolean} Is valid
 */
export const isValidGameUrl = (url) => {
    return url.startsWith("http") || url.startsWith("roadclient://");
};

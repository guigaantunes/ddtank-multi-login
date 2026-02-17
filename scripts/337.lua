-- Strategy: 337.com ddtank
-- Comment: Login via 337.com API

function login(username, password, server_id)
    local agent = agent()

    -- Step 0: Acessar página BR para estabelecer sessão na versão correta
    agent:get("https://web.337.com/pt/ddtank/?refer=1")

    -- Step 1: Login via API
    local login_url = string.format(
        "https://www.337.com/api.php?a=1002&username=%s&password=%s",
        username,
        password
    )
    
    local login_response = agent:get(login_url)
    
    -- Verificar se o login foi bem-sucedido
    local error_code = string.match(login_response, [["error":(%d+)]])
    if not error_code or error_code ~= "0" then
        error("Falha no login. Resposta: " .. login_response)
    end

    -- Step 2: Acessar página principal novamente para confirmar login
    local servers_page = agent:get("https://www.337.com/")
    
    -- Verificar se realmente está logado
    local logged_username = string.match(servers_page, [[<span class="e1">([^<]+)</span>]])
    if not logged_username or logged_username ~= username then
        error("Login não confirmado. Username esperado: " .. username .. ", encontrado: " .. tostring(logged_username))
    end
    
    -- Step 3: Gerar URL do servidor e acessar
    local play_url = string.format("https://www.337.com/play.php?id=%s", server_id)
    local play_page = agent:get(play_url)
    
    -- Extrair a URL do roadclient do JavaScript
    local game_url = string.match(play_page, [[window%.location%s*=%s*"(roadclient://[^"]+)"]])
    
    if not game_url then
        error("Falha ao extrair URL do jogo. A página pode ter mudado. Response size: " .. #play_page)
    end
    
    -- Abrir URL no navegador padrão do sistema (com roadclient://)
    os.execute('start "" "' .. game_url .. '"')

    return "Abrindo jogo no navegador..."
end

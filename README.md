# DDTank Multi-Login

Ferramenta de login multi-contas para DDTank, desenvolvida em Rust com interface grafica moderna usando Sciter. Permite gerenciar e alternar entre multiplas contas de forma rapida e pratica.

## Funcionalidades

### Gerenciamento de Contas
Adicione, edite e remova contas de DDTank com facilidade. Cada conta armazena usuario, senha, servidor e um apelido opcional para identificacao rapida. As contas sao salvas em um banco de dados local (`userdata.redb`), garantindo persistencia entre sessoes. NÂO COMPARTILHE ESSE ARQUIVO JAMAIS

### Login Automatizado
Ao clicar em uma conta, o sistema realiza o login automaticamente atraves de scripts Lua que simulam o processo de autenticacao no servidor 337.com. O login e feito em segundo plano e o jogo e aberto diretamente no logger que voce estiver usando.

### Multi-Servidor
Suporte a todos os servidores brasileiros do DDTank 337:
- Ilha dos Valentoes (S1-3, 9-10, 12-19)
- Vale dos Ouricos (S4-8, 11, 20-46)
- Jogos Olimpicos (S47-131, 362-375)
- Lugares Escuros (S132-394)
- Universo DDToker (S395-398)
- Legado dos Campeoes (S399)
- Aurora (S400)

### Busca de Contas
Campo de busca integrado que filtra contas em tempo real por nome de usuario ou apelido, facilitando a navegacao quando se tem muitas contas cadastradas.

### Ordenacao por Uso Recente
As contas sao automaticamente ordenadas pela ultima vez que foram utilizadas. A conta usada mais recentemente aparece primeiro na lista.

### Regua Integrada
Botao "Abrir Regua" que executa a ferramenta `reguinha.exe` (boomzruler) diretamente pela interface, util para medir distancias e calcular angulos durante o jogo.

### Encerramento Automatico de Processos
Ao fechar o aplicativo, todos os processos filhos (como a regua) sao encerrados automaticamente, evitando processos orfaos em execucao.

## Requisitos

- **sciter-js-sdk 5.0.2.7** (`sciter.dll`) - Runtime da interface grafica (ja carregada no build)

## Estrutura do Projeto

```
ddtank-rs/
├── src/
│   ├── main.rs          # Ponto de entrada, handler Sciter
│   ├── lib.rs           # Engine de banco de dados e estrategias
│   ├── ui/
│   │   ├── index.htm    # Pagina principal
│   │   ├── css/         # Estilos da interface
│   │   ├── js/          # Logica da interface (Reactor/JSX)
│   │   └── htm/         # Modais (adicionar/editar conta)
├── scripts/
│   └── 337.lua          # Script de login para 337.com
├── build.rs             # Script de build (empacotamento UI, copia de arquivos)
├── build-release.ps1    # Script para gerar release zipada
└── Cargo.toml           # Dependencias do projeto
```

## Como Usar

### Gerar release
```powershell
.\build-release.ps1
```
O script compila o projeto, copia os arquivos necessarios e gera um arquivo ZIP pronto para distribuicao.

## Tecnologias

- **Rust** - Linguagem principal
- **Sciter** - Framework de interface grafica (HTML/CSS/JS nativo)
- **Reactor** - Sistema reativo para UI (similar ao React)
- **redb** - Banco de dados embarcado em Rust
- **mlua** - Integracao Lua 5.4 em Rust
- **reqwest** - Cliente HTTP para login automatizado

## Contribuidores

<a href="https://github.com/guigaantunes/ddtank-multi-login/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=guigaantunes/ddtank-multi-login" />
</a>

## Licenca

Este projeto esta licenciado sob a licenca MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

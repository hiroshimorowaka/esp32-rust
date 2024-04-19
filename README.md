## Tabela de conteúdos
- [Tabela de conteúdos](#tabela-de-conteúdos)
- [Introdução](#introdução)
- [Contribuindo](#contribuindo)
- [Objetivo](#objetivo)
- [Hardware e Abstrações](#hardware-e-abstrações)
  - [Componentes usados](#componentes-usados)
  - [Display](#display)
  - [Botão de troca de estado](#botão-de-troca-de-estado)
  - [Botão de teste](#botão-de-teste)
  - [Leds](#leds)
  - [Diagrama](#diagrama)
- [Como rodar o projeto](#como-rodar-o-projeto)
  - [Requisitos globais](#requisitos-globais)
  - [WSL Only](#wsl-only)
  - [Windows](#windows)
  - [Requisitos locais e executando o projeto](#requisitos-locais-e-executando-o-projeto)

## Introdução

Esse projeto é uma maneira de treinar tanto Rust, uma linguagem de programação que eu gosto (mas sou bem iniciante), como também estudar sobre embarcados. 

Recentemente eu comprei um Esp32 (Mais especificamente o Esp32-wroom-32)

Esse repositório é onde eu vou compartilhar as minhas experiências e o código que eu estou estudando nesse Esp32.

## Contribuindo

Como dito acima, eu sou iniciante em Rust e em embarcados, então ta sendo uma experiência muito legal e desafiadora pra mim. Esse projeto é difícil contribuir com alguma coisa, mas eu criei ele pra vocês verem como eu construi o projeto e implementei as features.

A melhor coisa que pode ser feita para contribuir com o projeto é polir o código e compartilhar conhecimento tanto sobre Rust quanto sobre embarcados, já que sou iniciante e meu código é de amador mesmo. Seria muito legal criar uma discussão sobre e eu ficaria muito grato se vocês se dispusessem a contribuir com o código, melhorando ele e detalhando essas melhorias pra gerar conhecimento.  
Também é bom pra quem tiver interesse em embarcados mas nunca mexeu, molhar os dedos nesse mundão incrível.

Nesse projeto tem uma branch de HTTP, onde eu implemento um servidor HTTP pra controlar o display do ESP, mas ainda ta incompleto, então seria MUITO legal se alguém quisesse contribuir com isso também (mesmo não sendo o objetivo principal do projeto, o importante é gerar conhecimento e aprender mais, e com certeza isso será muito divertido).

## Objetivo

Esse projeto ta sendo feito com o objetivo de integrar com um módulo eletrônico artesanal feito pelo meu pai. Esse módulo vai se integrar com máquinas de corte a laser, e caso haja energia em um pino, ele vai mudar o modo da máquina pra X, e caso não, pra Y, utilizando um relê.

Então o código do Esp32 é bem simples, ele vai controlar esse estado, enviando ou não energia pra esse pino especifico com o apertar de um botão

A única "regra de negócio" é que a ativação e desativação desse pino não pode ser feita caso a máquina a laser esteja ligada, ou seja, o estado do pino (ligado ou desligado) precisa ser mantido caso a máquina esteja ligada, mesmo que o botão seja pressionado.

## Hardware e Abstrações

Nesse projeto, eu estou usando o Esp32, mais espeficiamente o Esp32-wroom-32 (NodeMCU Esp32S)

O projeto foi feito com no_std do Rust, ou seja, bare-metal -> sem sistema operacional rodando por baixo. 

Pra montar a base do projeto, foi usado o [esp-template](https://github.com/esp-rs/esp-template), um template da comunidade de Rust pra criar um projeto pronto e funcionando com todas as crates necessárias pra utilizar a maioria das funcionalidades do Esp32 de forma abstraida.

### Componentes usados
- 2 (dois) push buttons comuns, com resistor de 1k
- 1 (um) display SSD1306 (o que eu uso é LCD monocromático)
- 2 (dois) leds com resistor de 1k, um vermelho e um verde. (mas tanto faz a cor)

Meus botões são de dois pinos, mas caso o seu botão seja de quatro pinos, que é o mais comum, usem o diagrama que eu montei ali em baixo para se localizar melhor.

### Display
O display está ligado da seguinte forma:
- VCC -> 5V
- GND -> GND
- SCL -> P22 (Pin 22/GPIO 22) (Por padrão, esse é o pino SCL do Esp32)
- SDA -> P21 (Pin 21/GPIO 21) (Por padrão, esse é o pino SDA do Esp32)

O artigo que eu usei pra montar o montar o display na placa foi esse aqui:   
https://www.makerguides.com/how-to-connect-an-i2c-lcd-with-esp32/


### Botão de troca de estado
Ele está ligado da seguinte forma:
- Terminal 1 -> 3V3 
- Terminal 2 -> P14 (Pin 14/GPIO 14)

### Botão de teste 
Ele está ligado da seguinte forma:
- Terminal 1 -> 3V3 
- Terminal 2 -> P12 (Pin 12/GPIO 12)

O botão de teste é usado só pra simular um estado de "maquina ligada" (botão pressionado), onde a troca de estados NÃO pode ocorrer, ou "máquina desligada" (botão solto), onde a troca pode sim ocorrer.

### Leds
Eles estão ligados da seguinte forma:  

**LED VERDE**
- Negativo do led -> GND
- Positivo -> 16

**LED VERMELHO**
- Negativo do led -> GND
- Positivo -> 17

### Diagrama
![Diagrama](https://i.imgur.com/uZdpMpl.png)


## Como rodar o projeto

Eu rodo esse projeto dentro do WSL, então tudo que será ensinado aqui vale pra Linux nativo e WSL, dentro do Windows é praticamente a mesma coisa, só muda um passo extra.

### Requisitos globais

Primeiro, você precisa ter o Rust instalado com a ferramenta rustup   
https://www.rust-lang.org/pt-BR/learn/get-started

Agora precisa instalar as dependências do cargo

**Cargo**
```bash
cargo install cargo-generate
cargo install ldproxy
cargo install espup
cargo install espflash
cargo install cargo-espflash # Optional
```

Você precisa das seguintes libs (Linux)
```bash
# Debian/Ubuntu/etc.
apt-get install libudev-dev
# Fedora
dnf install systemd-devel
# Arch Linux
pacman -Syu base-devel
# a flag "yu" no pacman é opcional, mas evita erros de sync (aconteceu comigo)
```

Depois de ter instalado isso, precisamos instalar o espup, um toolchain para o Esp

```bash
espup install
```

Depois de instalado, vai aparecer um arquivo "export-esp.sh" na sua home, esse arquivo é necessário pra você conseguir usar o as ferramentas acima, ele vai setar as variaveis de ambiente necessárias. 

Pra ativar ele no seu terminal, basta usar o seguinte comando:
```bash
. $HOME/export-esp.sh
```

É recomandado você colocar esse comando dentro da config do seu terminal para ele ser executado toda vez que um novo é aberto.

### WSL Only
Agora se você usa o WSL como eu, você precisa de um passo extra. 
Por padrão, o WSL não reconhece dispositivos USB conectados na máquina HOST, então você precisa de uma ferramenta chamada `usbipd`

Para fazer o WSL reconhecer o seu Esp32, você precisa seguir as instruções da ferramenta nesse site da Microsoft: https://learn.microsoft.com/pt-br/windows/wsl/connect-usb

É bem simples de instalar e usar, mas caso algum problema ocorra, por favor, abram uma Issue.

### Windows
Se você ainda tem dúvidas ou usa Windows nativo como ambiente de desenvolvimento, por favor, leia as instruções no [The Rust on ESP Book](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html)


### Requisitos locais e executando o projeto

Após instalar todas as ferramentas necessárias, podemos clonar o projeto com `Git Clone` e executa-lo.

Pra rodar o projeto, primeiro vamos buildar ele pra ter certeza que está tudo certinho, usando o comando `cargo build`  
Depois disso, podemos rodar em modo debug com `cargo run` ou usando a ferramenta "espflash" que instalamos.  
O Cargo run não roda da mesma maneira que projetos tradicionais Rust, ele vai rodar um comando por baixo que na realidade é esse aqui:

```bash
cargo espflash flash --monitor
```

Caso você queira rodar o projeto em modo Release, em vez do modo Debug (que é o padrão), basta usar o seguinte comando:

```bash
cargo espflash flash --release --monitor
```

Para abrir o monitor serial, use somente o seguinte comando:

```bash
cargo espflash monitor
```

O comando `flash` vai compilar seu projeto e jogar o código dentro do seu Esp32 pra ser executado, o comando `monitor` abre o monitor serial, e caso você queira fazer os dois ao mesmo tmepo, basta usar `cargo espflash flash --monitor`

Vale a pena dar uma olhada na documentação do espflash caso tenham problemas ou interesse nos outros comandos: https://github.com/esp-rs/espflash/blob/main/cargo-espflash/README.md#usage
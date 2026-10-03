# Landing pública — `SITE`

As páginas públicas em `clinicore.com.br`, por onde quem não é cliente conhece o produto.

## FR-SITE-01 — Apresentação do produto

> Como visitante, quero entender o que o sistema faz e para quem, para que eu decida se ele serve à minha clínica.

O sistema deve publicar páginas indexáveis por buscadores que apresentam o produto a clínicas, redes e dentistas autônomos.

| Atributo | Valor |
| --- | --- |
| Justificativa | A landing é por onde quem não é cliente conhece o produto. |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SITE-01.1** — Dada a landing publicada, quando a página inicial é requisitada sem executar JavaScript, então o HTML da resposta contém o texto de apresentação.

## FR-SITE-02 — Entrada no sistema pela landing

> Como visitante, quero chegar às telas de entrar e de criar conta a partir da landing, para que eu comece a usar sem procurar outro endereço.

O sistema deve oferecer na landing os caminhos para entrar e para criar conta, que levam ao sistema em `app.clinicore.com.br`.

| Atributo | Valor |
| --- | --- |
| Justificativa | A landing e o sistema têm domínios distintos; o caminho entre eles precisa existir. |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SITE-02.1** — Dado um visitante na landing, quando escolhe criar conta, então chega à tela de cadastro do sistema.

## FR-SITE-03 — Captação de interessados

> Como visitante, quero deixar meu contato ou pedir uma demonstração, para que alguém me procure.

O sistema deve receber, por um formulário na landing, o contato de um interessado e entregá-lo ao decisor do produto.

| Atributo | Valor |
| --- | --- |
| Justificativa | A landing abre o contato entre o interessado e o decisor do produto. |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SITE-03.1** — Dado um visitante que envia o formulário preenchido, quando o envio é aceito, então o contato chega ao decisor do produto.

## FR-SITE-04 — Preços e planos

> Como visitante, quero ver os planos e os preços, para que eu saiba quanto custa antes de falar com alguém.

O sistema deve publicar na landing os planos e os preços do produto.

| Atributo | Valor |
| --- | --- |
| Justificativa | O visitante vê o preço sem falar com alguém. Depende do modelo comercial, ainda em aberto (OQ-08). |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SITE-04.1** — Dado o modelo comercial decidido, quando a página de preços é aberta, então mostra cada plano com o preço.

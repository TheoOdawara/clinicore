export interface MailMessage {
  subject: string;
  html: string;
  text: string;
}

function compose(
  subject: string,
  name: string,
  paragraphs: string[],
  action: string,
  link: string,
): MailMessage {
  const htmlEntities: Record<string, string> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  };
  const escapeHtml = (value: string): string =>
    value.replace(
      /[&<>"']/g,
      (character) => htmlEntities[character] ?? character,
    );
  const greeting = `Olá, ${name}.`;
  const html = [
    `<p>${escapeHtml(greeting)}</p>`,
    ...paragraphs.map((paragraph) => `<p>${escapeHtml(paragraph)}</p>`),
    `<p><a href="${escapeHtml(link)}" style="display:inline-block;padding:12px 20px;background:#0f766e;color:#ffffff;border-radius:6px;text-decoration:none">${escapeHtml(action)}</a></p>`,
  ].join("\n");
  const text = [greeting, ...paragraphs, `${action}: ${link}`].join("\n\n");

  return { subject, html, text };
}

export function verificationMail(name: string, link: string): MailMessage {
  return compose(
    "Confirme seu e-mail no Clinicore",
    name,
    [
      "Confirme seu e-mail para começar a usar o Clinicore.",
      "O link expira em 1 hora.",
    ],
    "Confirmar e-mail",
    link,
  );
}

export function passwordResetMail(name: string, link: string): MailMessage {
  return compose(
    "Redefinir sua senha do Clinicore",
    name,
    [
      "Recebemos um pedido para redefinir sua senha.",
      "O link expira em 1 hora.",
      "Se não foi você, ignore este e-mail.",
    ],
    "Redefinir senha",
    link,
  );
}

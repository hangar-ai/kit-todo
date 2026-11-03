//! Every word of the tab, in English and in French (`STRINGS` of
//! `tabs/todo/logic.js`). A count picks `key_one` or `key_other`.

use hangar_kit_sdk::i18n::Strings;

pub const EN: &[(&str, &str)] = &[
    ("tab", "TODO"),
    ("scan", "Scan"),
    ("scanAgain", "Scan again"),
    ("scanning", "Scanning…"),
    (
        "scanningText",
        "Reading the files of this workspace for TODO, FIXME, HACK and XXX comments.",
    ),
    ("stateFound_one", "{n} marker"),
    ("stateFound_other", "{n} markers"),
    ("stateNone", "Nothing to do"),
    ("lastScan", "Scanned {time}"),
    ("search", "Search a comment or a file"),
    ("groupLabel", "Group by"),
    ("byFile", "By file"),
    ("byTag", "By tag"),
    ("clearFilters", "Clear the filters"),
    ("open", "Open"),
    ("openLabel", "Open {file}"),
    ("handOver", "Hand to an agent"),
    ("handingOver", "Sending…"),
    ("sentTo", "Sent to {agent}"),
    ("agentLabel", "Agent"),
    (
        "noAgent",
        "No Claude Code agent can receive text in this workspace.",
    ),
    (
        "panesDenied",
        "This tab may not list the agents, so it cannot hand a comment over. Accept panes.read in Settings › Kits.",
    ),
    ("author", "Who wrote it"),
    (
        "authorLabel",
        "Show who wrote the markers of {file}, and when",
    ),
    ("authorLoading", "Reading git…"),
    ("authorBy", "{author}, {time}"),
    ("authorNotCommitted", "Not committed yet"),
    (
        "blameStatus_untracked",
        "Git does not know this file yet, so there is no author.",
    ),
    (
        "blameStatus_no_git",
        "This folder is not a git repository, or git is not installed.",
    ),
    ("blameStatus_error", "Git could not blame this file."),
    (
        "blameDenied",
        "This tab may not run the scan command. Accept command:scan in Settings › Kits.",
    ),
    ("showMore", "Show {n} more"),
    ("noText", "No text after the tag"),
    ("matches_one", "{n} of {total} marker"),
    ("matches_other", "{n} of {total} markers"),
    ("tooManyTitle", "Too many to list"),
    (
        "tooMany",
        "{total} markers found; the first {shown} are listed. The counts above are complete. Search or filter, or fix some and scan again.",
    ),
    (
        "incomplete",
        "The scan stopped before the end of the folder (too many files or too slow). Add what you do not want read to .gitignore.",
    ),
    ("noneTitle", "No TODO in this workspace"),
    (
        "noneText",
        "No TODO, FIXME, HACK or XXX comment was found in the files that are not ignored. Only comments that start with the tag count, not the word in prose.",
    ),
    ("nothingMatches", "Nothing matches"),
    (
        "nothingMatchesText",
        "No marker matches this search and these tags.",
    ),
    ("deniedTitle", "This tab may not run the scan"),
    (
        "deniedText",
        "Accept command:scan in Settings › Kits, then try again. The tab only runs the scan script it declares, and only reads files.",
    ),
    ("errorTitle", "The scan did not finish"),
    (
        "error.timeout",
        "It took longer than expected. Try again, or add the heavy folders to .gitignore.",
    ),
    (
        "error.not_found",
        "Node.js was not found on this computer. Install it, then try again.",
    ),
    (
        "error.busy",
        "Hangar is already running the most commands allowed. Try again in a moment.",
    ),
    (
        "error.unreadable",
        "The script printed something that is not a scan. Reload the tab, then try again.",
    ),
    ("error.failed", "The script could not run: {detail}"),
    ("retry", "Try again"),
    (
        "notSaved",
        "This scan is shown but was not kept for the next time the tab opens: {detail}",
    ),
    (
        "savedPartly",
        "Only the first {kept} of the {shown} markers listed were kept for the next time the tab opens: the scan is too large to keep whole.",
    ),
    (
        "problem.denied",
        "This tab may not write to agents. Accept panes.send in Settings › Kits.",
    ),
    (
        "problem.not_found",
        "{agent} is no longer in this workspace.",
    ),
    (
        "problem.rate_limited",
        "Too many sends in a minute. Try again in a moment.",
    ),
    (
        "problem.unavailable",
        "{agent} cannot receive text right now. Pick another agent, or try again.",
    ),
    ("problem.failed", "The send failed: {detail}"),
    ("problem.open", "The file could not be opened: {detail}"),
    (
        "problem.open_denied",
        "This tab may not open files. Accept preview.show in Settings › Kits.",
    ),
];

pub const FR: &[(&str, &str)] = &[
    ("tab", "TODO"),
    ("scan", "Analyser"),
    ("scanAgain", "Analyser de nouveau"),
    ("scanning", "Analyse en cours…"),
    (
        "scanningText",
        "Lecture des fichiers de ce workspace à la recherche des commentaires TODO, FIXME, HACK et XXX.",
    ),
    ("stateFound_one", "{n} marqueur"),
    ("stateFound_other", "{n} marqueurs"),
    ("stateNone", "Rien à faire"),
    ("lastScan", "Analysé {time}"),
    ("search", "Chercher un commentaire ou un fichier"),
    ("groupLabel", "Grouper par"),
    ("byFile", "Par fichier"),
    ("byTag", "Par étiquette"),
    ("clearFilters", "Effacer les filtres"),
    ("open", "Ouvrir"),
    ("openLabel", "Ouvrir {file}"),
    ("handOver", "Confier à un agent"),
    ("handingOver", "Envoi…"),
    ("sentTo", "Envoyé à {agent}"),
    ("agentLabel", "Agent"),
    (
        "noAgent",
        "Aucun agent Claude Code ne peut recevoir de texte dans ce workspace.",
    ),
    (
        "panesDenied",
        "Cet onglet n’a pas le droit de lister les agents : il ne peut donc rien leur confier. Accepte panes.read dans Réglages › Kits.",
    ),
    ("author", "Qui l’a écrit"),
    (
        "authorLabel",
        "Voir qui a écrit les marqueurs de {file}, et quand",
    ),
    ("authorLoading", "Lecture de git…"),
    ("authorBy", "{author}, {time}"),
    ("authorNotCommitted", "Pas encore commité"),
    (
        "blameStatus_untracked",
        "Git ne connaît pas encore ce fichier : pas d’auteur.",
    ),
    (
        "blameStatus_no_git",
        "Ce dossier n’est pas un dépôt git, ou git n’est pas installé.",
    ),
    ("blameStatus_error", "Git n’a pas pu analyser ce fichier."),
    (
        "blameDenied",
        "Cet onglet n’a pas le droit de lancer la commande d’analyse. Accepte command:scan dans Réglages › Kits.",
    ),
    ("showMore", "En afficher {n} de plus"),
    ("noText", "Aucun texte après l’étiquette"),
    ("matches_one", "{n} sur {total} marqueur"),
    ("matches_other", "{n} sur {total} marqueurs"),
    ("tooManyTitle", "Trop de marqueurs pour tout lister"),
    (
        "tooMany",
        "{total} marqueurs trouvés ; les {shown} premiers sont listés. Les totaux ci-dessus sont complets. Cherche, filtre, ou corrige-en quelques-uns puis relance l’analyse.",
    ),
    (
        "incomplete",
        "L’analyse s’est arrêtée avant la fin du dossier (trop de fichiers ou trop lent). Ajoute à .gitignore ce que tu ne veux pas lire.",
    ),
    ("noneTitle", "Aucun TODO dans ce workspace"),
    (
        "noneText",
        "Aucun commentaire TODO, FIXME, HACK ou XXX dans les fichiers non ignorés. Seuls comptent les commentaires qui commencent par l’étiquette, pas le mot dans une phrase.",
    ),
    ("nothingMatches", "Aucun résultat"),
    (
        "nothingMatchesText",
        "Aucun marqueur ne correspond à cette recherche et à ces étiquettes.",
    ),
    (
        "deniedTitle",
        "Cet onglet n’a pas le droit de lancer l’analyse",
    ),
    (
        "deniedText",
        "Accepte command:scan dans Réglages › Kits, puis réessaie. L’onglet ne lance que le script d’analyse qu’il déclare, et ne fait que lire des fichiers.",
    ),
    ("errorTitle", "L’analyse ne s’est pas terminée"),
    (
        "error.timeout",
        "Elle a pris plus de temps que prévu. Réessaie, ou ajoute les gros dossiers à .gitignore.",
    ),
    (
        "error.not_found",
        "Node.js est introuvable sur cet ordinateur. Installe-le, puis réessaie.",
    ),
    (
        "error.busy",
        "Hangar lance déjà le maximum de commandes autorisé. Réessaie dans un instant.",
    ),
    (
        "error.unreadable",
        "Le script a affiché autre chose qu’une analyse. Recharge l’onglet, puis réessaie.",
    ),
    ("error.failed", "Le script n’a pas pu se lancer : {detail}"),
    ("retry", "Réessayer"),
    (
        "notSaved",
        "Cette analyse est affichée mais n’a pas été gardée pour la prochaine ouverture de l’onglet : {detail}",
    ),
    (
        "savedPartly",
        "Seuls les {kept} premiers des {shown} marqueurs listés ont été gardés pour la prochaine ouverture de l’onglet : l’analyse est trop grosse pour être gardée en entier.",
    ),
    (
        "problem.denied",
        "Cet onglet n’a pas le droit d’écrire aux agents. Accepte panes.send dans Réglages › Kits.",
    ),
    ("problem.not_found", "{agent} n’est plus dans ce workspace."),
    (
        "problem.rate_limited",
        "Trop d’envois en une minute. Réessaie dans un instant.",
    ),
    (
        "problem.unavailable",
        "{agent} ne peut pas recevoir de texte pour l’instant. Choisis un autre agent, ou réessaie.",
    ),
    ("problem.failed", "L’envoi a échoué : {detail}"),
    ("problem.open", "Le fichier n’a pas pu s’ouvrir : {detail}"),
    (
        "problem.open_denied",
        "Cet onglet n’a pas le droit d’ouvrir des fichiers. Accepte preview.show dans Réglages › Kits.",
    ),
];

/// The tab's words, spoken in `locale` (`en` or `fr`; English for any other).
pub fn strings(locale: &str) -> Strings {
    Strings::new(locale).add("en", EN).add("fr", FR)
}

# Relazione Progetto: GitCrab

## Introduzione

GitCrab è un'applicazione web sviluppata in Rust utilizzando il framework Loco.rs, ispirato a Ruby on Rails. Questo progetto si basa sul pattern architetturale MVC (Model-View-Controller), progettato per gestire repository personali in un modo simile a GitHub.
![alt text](<Screenshot1.png>)
### Obiettivi del Progetto

L'obiettivo di GitCrab è offrire una piattaforma per la gestione efficiente e sicura di repository, permettendo agli utenti di:

- Registrarsi e autenticarsi tramite un sistema completo.
- Creare e gestire repository con funzionalità complete CRUD.
- Gestire chiavi SSH per il potenziamento della sicurezza.
- Interagire con i propri repository tramite comandi Git in un ambiente SSH integrato.

## Pattern MVC

Il pattern MVC divide l'applicazione in tre componenti principali:

- **Model**: Gestisce i dati e la logica dell'applicazione. In GitCrab, i modelli rappresentano entità come `Repository` e `User`.
- **View**: Rappresenta la parte visiva dell'applicazione, gestendo i layout e la presentazione dei dati. Utilizza template HTML per rendere le interfacce utente.
- **Controller**: Funziona da intermediario tra Model e View, gestendo la logica di business e le interazioni dell'utente. I controller di GitCrab elaborano le richieste e aggiornano i modelli e le viste di conseguenza.

## Caratteristiche Principali

### 1. **Sistema di Autenticazione**
- **Registrazione e login**: Implementazione di un sistema di autenticazione con gestione delle sessioni tramite JWT.
- **Logout**: Interfaccia dedicata per disconnettersi.

### 2. **Gestione Repository**
- **CRUD**: Funzionalità per creare, leggere, aggiornare e cancellare repository, ognuno con un campo “nome”.

### 3. **Gestione Chiavi SSH**
- **CRUD Chiavi SSH**: Funzione per gestire chiavi SSH, includendo titoli e chiavi pubbliche.

### 4. **Server SSH Integrato**
- Gestione dei comandi `git-upload-pack` e `git-receive-pack` per operazioni git pull e push tramite SSH.

## Infrastruttura

- **Persistenza dei dati**: Utilizzo di PostgreSQL 16 per la gestione persistente dei dati.
- **Containerizzazione**: Utilizzo di Docker per il deploy dell'applicazione.

# Docker Compose

Il sistema di containerizzazione per GitCrab è gestito tramite **Docker Compose**, che semplifica la configurazione e il lancio di diversi servizi necessari per l'applicazione. Di seguito è riportata una spiegazione dettagliata del file `docker-compose.yml` utilizzato nel progetto.

## Servizi

Il file definisce due servizi principali: `db` per il database PostgreSQL e `app` per l'applicazione GitCrab.

### Servizio Database (`db`)

- **Immagine:** Utilizza l'immagine `postgres:15.3-alpine` per un ambiente di database leggero e ottimizzato.

- **Nome del Container:** `gitcrab-db`

- **Variabili d'Ambiente:**

  - `POSTGRES_USER`: Nome utente per accedere al database (`loco`).

  - `POSTGRES_PASSWORD`: Password per l'utente (`loco`).

  - `POSTGRES_DB`: Nome del database utilizzato dall'applicazione (`gitcrab_development`).

- **Volume:** Consente la persistenza dei dati su disco attraverso un volume Docker chiamato `postgres_data`, assicurando che i dati non vengano persi quando il container viene riavviato.

- **Porta:** Esporta la porta `5432` per consentire le connessioni al database.

- **Controllo Salute:** Include un controllo della salute che utilizza il comando `pg_isready` per verificare la disponibilità del database.

- **Rete:** Collega il servizio a una rete Docker chiamata `db`.

### Servizio Applicazione (`app`)

- **Build:** Specifica il contesto di costruzione e il Dockerfile da utilizzare per compilare l'immagine dell'applicazione.

- **Nome del Container:** `gitcrab-app`

- **Dipendenze:** Il servizio `app` dipende dal servizio `db` e non verrà avviato finché `db` non è in uno stato sano.

- **Variabili d'Ambiente:**

  - `DATABASE_URL`: URL di connessione al database configurato per l'applicazione.

- **Porta:** Esporta la porta `5150` per l'applicazione e la porta `22` per le operazioni SSH necessarie per Git.

- **Volumi:**

  - Monta diverse directory dal file system locale ai percorsi appropriati nel container, assicurando la persistenza e l'accesso ai file di configurazione, asset e repository Git.

- **Rete:** Anche il servizio `app` è collegato alla rete `db`.

- **File di Ambiente:** Il servizio carica le variabili d'ambiente da un file `.env`, semplificando la gestione della configurazione.

## Volumi

Il file Docker Compose definisce tre volumi per la persistenza dei dati:

- `postgres_data`: utilizzato per memorizzare i dati del database PostgreSQL.

- `git_repositories`: utilizzato per conservare i repository Git.

- `git_ssh_keys`: utilizzato per conservare le chiavi SSH necessarie per le operazioni Git.

## Reti

Viene creata una rete Docker chiamata `db`, che consente una comunicazione sicura e efficiente tra i servizi `db` e `app`.

# Server SSH

Il server SSH integrato in GitCrab è stato progettato per offrire un’esperienza Git nativa e sicura, consentendo agli utenti di eseguire comandi `git pull` e `git push` su repository privati senza rinunciare alla semplicità operativa.

## Architettura e Avvio

All’interno del container dell’applicazione, il demone SSH (`sshd`) viene avviato in background con una configurazione dedicata. L’utente di sistema “git” è predisposto con una shell limitata che punta a uno script di controllo (git-serve). In questo modo ogni connessione SSH viene intercettata e gestita unicamente attraverso un unico punto di ingresso, evitando accessi shell interattivi non autorizzati.

## Gestione di Permessi e Sicurezza

- **Utente dedicato**: tutte le operazioni vengono eseguite con l’UID “git”, separando nettamente i permessi delle risorse di sistema da quelli dell’applicativo.  

- **Directory protette**:  

  - `/home/git/.ssh` e il file `authorized_keys` hanno permessi restrittivi (700/600), per impedire modifiche non autorizzate.  

  - I repository sono montati su un volume dedicato, con permessi 755, garantendo accesso controllato sia in lettura sia in scrittura.  

- **Ambiente isolato**: la shell personalizzata impedisce l’esecuzione di comandi diversi da quelli Git standard, riducendo la superficie di attacco.

## Dispatch dei Comandi Git

Ogni volta che un client invia un comando SSH, `sshd` passa il controllo allo script di dispatch:

1. **Validazione del comando**  

   Il sistema verifica che l’istruzione ricevuta sia esattamente `git-upload-pack` (pull) o `git-receive-pack` (push). Tutti gli altri comandi vengono scartati, proteggendo il server da operazioni non previste.

2. **Parsing del repository**  

   Il nome del repository viene estratto con logiche minimali di parsing, garantendo robustezza anche in presenza di path complesse o varianti di citazione.

3. **Controllo di esistenza**  

   Prima di avviare l’effettivo comando Git, viene verificata la presenza della directory e di un file `HEAD`. Solo in caso di repository valido si prosegue con l’operazione richiesta.

4. **Esecuzione dei comandi “plumbing”**  

   Lo script chiama direttamente i comandi di basso livello `git-upload-pack` e `git-receive-pack`. Questo approccio preserva la compatibilità con tutti i client Git esistenti e utilizza il motore interno di Git per gestire la replica dei dati.

## Logging e Tracciabilità

Tutte le operazioni—successi, errori o comandi non autorizzati—vengono registrate su un file di log centrale. Questo consente di:

- Monitorare in tempo reale chi accede a quali repository.

- Eseguire audit di sicurezza e risalire a eventuali tentativi di accesso non legittimi.

- Analizzare statistiche di utilizzo e individuare colli di bottiglia o anomalie.

## Vantaggi del Design

- **Sicurezza**: l’uso di un’utenza dedicata con shell limitata e permessi stretti riduce i rischi di compromissione.  

- **Manutenibilità**: uno script unico per il dispatch semplifica l’evoluzione delle policy di accesso e l’introduzione di nuove funzionalità.  

- **Compatibilità**: invocando i comandi Git ufficiali, il server resta pienamente interoperabile con qualsiasi client Git, senza richiedere plugin o estensioni.  

- **Scalabilità**: la separazione tra servizio SSH e logica applicativa permette di replicare o bilanciare il carico dei comandi Git in ambienti cluster.

---

**In sintesi**, il server SSH di GitCrab unisce sicurezza, trasparenza e piena compati

# Autenticazione

L'autenticazione in GitCrab è progettata per garantire una gestione sicura degli accessi degli utenti, utilizzando un approccio basato su **JSON Web Token (JWT)** e cookie per mantenere le sessioni.

## Design dell'Autenticazione

1. **Registrazione dell'Utente**:

   - Gli utenti possono registrarsi fornendo un nome utente, un'email e una password nel modulo di registrazione. Questi dati vengono inviati all'endpoint `/api/auth/register`.

   - La registrazione è implementata nel seguente modo:

     - Viene verificata l'unicità dell'email fornita.

     - Se l'email è valida, viene creato un nuovo utente nel database con le credenziali fornite, la password fornita viene salva is hash per motivi di sicurezza.

     - Successivamente, viene inviata un'email di benvenuto all'utente utilizzando Mailpit, un sistema di gestione della posta elettronica locale. Ciò consente di testare e verificare le funzionalità di invio email senza la necessità di un servizio di invio email esterno.

2. **Login dell'Utente**:

   - Gli utenti registrati possono accedere utilizzando il loro indirizzo email e la password forniti durante la registrazione. La richiesta di login viene inviata all'endpoint `/api/auth/login`.

   - Durante il processo di login, l'email e la password vengono verificate. Se le credenziali sono corrette, il server genera un token JWT.

3. **Utilizzo di JWT e Cookie**:

   - Il token JWT generato viene inviato al client e memorizzato nei cookie. Questo approccio permette di mantenere la sessione dell'utente anche dopo che il browser viene chiuso, grazie all'uso di cookie persistenti.

   - L'header del cookie include proprietà di sicurezza come `Secure` e `HttpOnly`, per ridurre il rischio di attacchi XSS e CSRF.

4. **Gestione della Sessione**:

   - Quando l'utente effettua il login, il token JWT viene utilizzato per autenticare ulteriori richieste. Il server verifica il token per garantire che sia valido e non sia scaduto.

   - La sessione dell'utente rimane attiva finché il token è valido, offrendo un accesso sicuro a tutte le operazioni consentite.

- **Limitazione**
  - Per ridurre la complessita del sistema in particolare per la gestione dell'accesso tramite ssh per reoisitory, si e' scelto di non introdure us sitema di sessione individuale per utente, quindi ogni utente registrato avra' accesso all'intero roster di repository

## Implementazione del Sistema di Email

La logica di invio delle email è gestita dal modulo `mailers::auth::AuthMailer`, che utilizza Mailpit per inviare email di benvenuto e altre comunicazioni necessarie agli utenti. Questo approccio consente agli sviluppatori di testare l'invio senza rischiare di inviare errori a utenti reali.

# Gestione dei Repository

La gestione dei repository in GitCrab è uno dei pilastri centrali del progetto. È implementata mediante architetture e design coerenti, integrando sia funzionalità per la manipolazione dei repository sul filesystem sia un’interfaccia intuitiva per interagire con essi.

![alt text](<Screenshot2.png>)

---

## Design della Gestione dei Repository

### Obiettivi Principali

1. **CRUD Completo**: Consente la creazione, lettura, aggiornamento e cancellazione dei repository.

2. **Sincronizzazione con il Filesystem**: Ogni operazione viene eseguita e tracciata sia sul database che sul filesystem utilizzando il servizio personalizzato `GitService`.

3. **Affidabilità**: Le operazioni includono rollback automatici in caso di errore, garantendo consistenza tra il database e il filesystem.

4. **Compatibilità con Git**: Ogni repository viene inizializzato come bare repository per supportare le operazioni standard di `git clone`, `git pull` e `git push`.

---

## Funzionamento del Sistema

### Architettura Generale

L'architettura è suddivisa in componenti chiave:

- **Controller Git**: Esporta endpoint pubblici per gestire le azioni sui repository (e.g., creare, aggiornare, eliminare).

- **Modello Entity**: Gestisce la rappresentazione dei repository all’interno del database.

- **GitService**: Un modulo dedicato che interagisce con il filesystem per creare, rinominare, eliminare o spostare i repository.

### Dettaglio sui Processi Operativi

#### 1. **Creazione di Repository**

- Tramite un form o una richiesta API, l’utente fornisce il nome del repository desiderato.

- Il servizio `GitService` utilizza il comando `git init --bare` per creare un repository vuoto nella directory di base configurata.

- La directory viene protetta con permessi appropriati e il database viene aggiornato inserendo i metadati del repository.

#### 2. **Aggiornamento di Repository**

- Il nome di un repository può essere modificato dall’utente. Viene verificata l’assenza di conflitti con altri repository esistenti.

- Il nome del repository viene aggiornato nel database e la directory sul filesystem viene rinominata di conseguenza.

- In caso di errore nella sincronizzazione (e.g., un problema durante l’aggiornamento del database), il sistema esegue un rollback per ripristinare la situazione originale.

#### 3. **Eliminazione di Repository**

- Quando un repository viene eliminato, il sistema assicura la rimozione sia dal database sia dal filesystem. Tutti i dati e i file associati al repository vengono rimossi definitivamente.

#### 4. **Visualizzazione dei Repository**

- Gli utenti possono accedere a una lista dei repository disponibili, che viene recuperata ordinando i record dal database.

- È possibile visualizzare dettagli specifici sui repository, come data di creazione, percorso sul filesystem e nome.

---

## Sicurezza e Validazione

### Controllo dei Permessi

- Tutte le operazioni sui repository sono protette da un middleware di autenticazione basato su JWT. Solo utenti autenticati possono accedere alle funzionalità di manipolazione.

- Le directory dei repository sul filesystem seguono permessi rigidi per limitare l’accesso non autorizzato.

### Validazione dei Nomi

- I nomi dei repository vengono sanitizzati per evitare caratteri non validi o potenzialmente dannosi. Solo caratteri alfanumerici, trattini e underscore sono consentiti.

- Nomi duplicati non sono ammessi.

---

## Rollback e Recupero da Errori

Ogni operazione sul filesystem è progettata per includere una logica di rollback. Ad esempio:

1. Se un aggiornamento del nome viene completato sul filesystem ma non sul database, viene eseguita un’inversione del nome sul filesystem.

2. Se una creazione di repository fallisce a livello del database, i file temporanei sul filesystem vengono ripuliti automaticamente.

---

## Vantaggi del Design

1. **Robustezza**: Grazie al rollback automatico e al controllo delle operazioni, il sistema garantisce consistenza dei dati anche in presenza di errori.

2. **Scalabilità**: Supportando repository bare e utilizzando un design modulare (controller → servizio → filesystem), la gestione può essere estesa a cluster o ambienti distribuiti.

3. **Facilità d’Uso**: L’interfaccia intuitiva semplifica la gestione dei repository, pur nascondendo complessità operative.

# Gestione delle chiavi SSH

## Introduzione

La gestione delle chiavi SSH è un aspetto fondamentale per assicurare un accesso sicuro degli utenti alle risorse di un server. Questo sistema è progettato con i principi di scalabilità, robustezza e sicurezza avanzata, integrandosi strettamente con il filesystem e i database. Il cuore del funzionamento si basa su un servizio chiamato `SshKeyService` e diverse API HTTP. Questo documento spiega il funzionamento e le motivazioni dietro la progettazione di questa architettura.
![alt text](<Screenshot3.png>)

---

## Architettura e funzionamento

### 1. **Componente centrale: SshKeyService**

Il cuore della gestione delle chiavi SSH è una classe denominata `SshKeyService`. Il suo obiettivo principale è gestire il file `authorized_keys`, che contiene tutte le chiavi SSH autorizzate per il server. Questo file si trova solitamente nel percorso `~/.ssh/authorized_keys`.

Ogni operazione di gestione delle chiavi è costruita attorno a tre metodi principali:

- **`add_key`**: Aggiunge una nuova chiave pubblica al file `authorized_keys`.

- **`remove_key`**: Rimuove una chiave esistente dal file.

- **`update_key`**: Combina le funzioni di rimozione e aggiunta per sostituire una vecchia chiave con una nuova.

Queste operazioni utilizzano meccanismi robusti per l'accesso e la manipolazione dei file, garantendo che non si verifichino interruzioni o corruzioni.

#### Perché questa progettazione?

- *Chiarezza e modularità*: Ogni funzione ha una responsabilità unica, rendendo il codice più semplice da mantenere.

- *Sicurezza*: La manipolazione diretta del file `authorized_keys` è incapsulata, riducendo il rischio di errori.

- *Atomicità*: Le operazioni, come l'aggiunta o la rimozione di chiavi, sono progettate per essere eseguite per intero, evitando stati parziali.

---

### 2. **Flussi operativi basati su HTTP**

Il sistema espone una API HTTP per consentire una gestione più semplice tramite interfaccia web o script esterni. I punti endpoint API supportano operazioni come:

- **Creare una nuova chiave SSH** tramite `/sshes/new` o `/sshes/add`.

- **Modificare una chiave esistente**, riferendosi all'ID associato (es. `update`).

- **Elencare** e visualizzare le chiavi attuali.

#### Esempio operativo:

Quando si aggiunge una chiave:

1. L'utente invia un modulo (`Form`) contenente la chiave pubblica e un titolo descrittivo.

2. La chiave viene validata e salvata nel database.

3. Il metodo `add_key` aggiorna il file `authorized_keys` aggiungendo la nuova chiave.

4. Ogni passaggio è tracciato con un sistema di log (journaling).

#### Perché un'API HTTP?

- *Interoperabilità*: Consente una facile interazione con altri strumenti e sistemi.

- *Scalabilità*: Facilita l'integrazione in sistemi distribuiti o pipeline CI/CD.

- *Centralizzazione*: Fornisce un punto di gestione unico e centralizzato per gli utenti e le chiavi.

---

### 3. **Integrazione con il database**

Ogni chiave SSH è registrata in un database come modello `Entity`. Questo approccio offre diversi vantaggi:

- Mantiene una *fonte di verità* centralizzata, facilitando il controllo degli accessi.

- Consente di monitorare il ciclo di vita delle chiavi, incluse le date di aggiunta e rimozione.

- Collega le chiavi a utenti o sistemi specifici, tramite metadati come il titolo.

#### Esempio:

Quando una chiave viene aggiunta, si eseguono due operazioni sincronizzate:

1. Viene inserita un'entry nel database tramite il modello `ActiveModel`.

2. Viene sincronizzata nel filesystem aggiornando il file `authorized_keys`.

#### Perché questa integrazione?

- *Sincronizzazione dei dati*: Riduce al minimo gli errori di gestione grazie a una stretta coordinazione tra backend e livello di sistema.

- *Tracciabilità*: Fornisce un registro completo per ogni chiave SSH aggiunta o rimossa.

---

### 4. **Sicurezza e robustezza**

La sicurezza è fondamentale nella gestione delle chiavi SSH. Alcune delle principali caratteristiche includono:

- Ogni chiave inviata tramite HTTP è **convalidata** (per formato e unicità).

- L'accesso al file `~/.ssh/authorized_keys` è strettamente controllato per impedire accessi non autorizzati.

- Tutte le operazioni sono **registrate** tramite log livello `INFO` e `ERROR`, garantendo l'audit completo.

- Le eccezioni (`Error::Message`) gestiscono eventuali anomalie (es. problemi nella scrittura del file) senza compromettere il sistema.

#### Perché tanta enfasi sulla sicurezza?

- Il file `authorized_keys` è critico per l'accesso al server e persino una minima corruzione può compromettere la sicurezza.

- La progettazione previene attacchi di tipo injection o malfunzionamenti dovuti a sovraccarichi.

---

## Decisioni di progettazione

1. **Encapsulamento con `SshKeyService`:**

   - Fornisce un'astrazione tra la logica applicativa e la gestione tecnica sottostante.

   - Facilita l'espansione futura e i test unitari.

2. **Atomicità delle operazioni:**

   - Previene condizioni di corsa (race conditions) in ambienti multi-thread o distribuiti.

3. **Uso di Axum e SeaORM:**

   - Garantisce performance elevate con una sintassi chiara e type-safe in Rust.

   - Axum struttura middleware e API HTTP in modo efficace.

4. **Database e filesystem combinati:**

   - Centralizza la visibilità e l'integrità delle chiavi gestite.

   - Crea un sistema auto-correttivo grazie al rollback transazionale.


# Pattern MVC in GitCrab

## Introduzione al Pattern MVC

Il pattern **Model-View-Controller (MVC)** è un paradigma architetturale che suddivide un'applicazione in tre componenti principali: *Model*, *View* e *Controller*. Questa suddivisione consente di separare efficacemente le responsabilità, favorendo modularità, manutenibilità e riutilizzabilità del codice.

GitCrab adotta il pattern MVC come fondamento strutturale della sua architettura per gestire sia il backend che l'interfaccia utente. Sviluppato in Rust usando il framework **Loco.rs** (ispirato a Ruby on Rails), l'utilizzo di MVC si dimostra ideale per raggiungere scalabilità ed efficienza, mantenendo alta la qualità del software attraverso le migliori pratiche di progettazione.

---

## Componenti del Pattern MVC

### 1. **Model**

Il _Model_ è responsabile della gestione dei dati e rappresenta la fonte di verità del sistema. Si occupa di interfacciarsi con il database e di rappresentare logicamente le entità principali di GitCrab, come repository, utenti e chiavi SSH.

#### Funzioni chiave:
- Permette la manipolazione dei dati persistenti utilizzando il pacchetto ORM **SeaORM**, che fornisce un'astrazione di alto livello per il database.
- Gestisce operazioni di validazione e trasformazione dei dati prima che questi vengano salvati o recuperati dal database.
- Facilita la sincronizzazione tra i dati memorizzati nel database e il filesystem, specialmente per entità come repository Git e chiavi SSH.

#### Implementazione:
Esempio: Il modello `SshKey` rappresenta una chiave SSH nel database. Di seguito una possibile definizione di questa entità:

struct SshKey {
    id: i32,
    title: String,
    public_key: String,
    created_at: DateTime,
}

Grazie a **SeaORM**, le operazioni database-agnostiche possono essere eseguite in modo type-safe, migliorando affidabilità e leggibilità del codice.

#### Punti di forza:
- **Astrazione del database**: Il codice si concentra sulla logica applicativa, evitando manipolazioni SQL dirette.
- **Consistenza centralizzata**: La validazione e la logica aziendale possono essere applicate uniformemente.

#### Debolezze:
- **Rischio di complessità eccessiva**: Con modelli troppo complessi, le prestazioni potrebbero risentirne, specialmente in presenza di query pesanti.
- **Curva di apprendimento**: Rust e un ORM come **SeaORM** richiedono capacità tecniche avanzate per sfruttarne appieno il potenziale.

---

### 2. **View**

Il componente _View_ gestisce la renderizzazione e la presentazione dei dati. In GitCrab, l'interfaccia utente viene implementata tramite **Tera**, un motore di template che separa la logica di presentazione dalla logica di business.

#### Funzioni chiave:
- Visualizza dati formattati in modo leggibile ed esteticamente chiaro, come la lista dei repository o il pannello di gestione delle chiavi SSH.
- Integra dinamicamente dati provenienti dai Controller, che passano variabili utili al rendering di layout HTML.

#### Implementazione:
I template HTML sono archiviati in directory specifiche e suddivisi per funzionalità. Esempio: un template dedicato alla visualizzazione chiavi SSH (`ssh/list.html`) potrebbe essere strutturato come segue:

```tera
<table>
    <tr>
        <th>Titolo</th>
        <th>Chiave Pubblica</th>
        <th>Creata il</th>
    </tr>
    {% for key in ssh_keys %}
        <tr>
            <td>{{ key.title }}</td>
            <td>{{ key.public_key }}</td>
            <td>{{ key.created_at | date }}</td>
        </tr>
    {% endfor %}
</table>
```

#### Punti di forza:
- **Separazione della presentazione dalla logica**: Le View trasformano i dati senza dover conoscere i dettagli della logica di business.
- **Personalizzazione semplice**: I layout HTML possono essere modificati senza intaccare il resto del sistema.

#### Debolezze:
- **Mancanza di dinamicità client-side**: Le View completamente server-rendered potrebbero risultare più lente rispetto a framework front-end moderni come React.
- **Carico sul backend**: Ogni aggiornamento del contenuto richiede una comunicazione server-side.

---

### 3. **Controller**

Il _Controller_ agisce da intermediario tra il _Model_ e la _View_, orchestrando la gestione delle richieste dell'utente e la fornitura di risposte appropriate. In GitCrab, i controller sono implementati per gestire endpoint HTTP definiti e ricevere parametri dinamici.

#### Funzioni chiave:
- Instrada le richieste HTTP a metodi appropriati, utilizzando il potente sistema di routing di Axum.
- Valida e pre-elabora i dati ricevuti prima di passarli al Model.
- Restituisce View renderizzate o altri tipi di risposta (ad esempio file JSON per le API).

#### Implementazione:
Esempio di metodo controller per aggiungere una chiave SSH:
```rust
async fn add(
    State(ctx): State<AppContext>,
    Form(params): Form<SshKeyParams>,
) -> Result<Redirect> {
    let new_key = ActiveModel {
        title: Set(params.title),
        public_key: Set(params.public_key),
        ..Default::default()
    };
    let saved_key = new_key.insert(&ctx.db).await?;
    Ok(Redirect::to("/sshes"))
}
```

#### Punti di forza:
- **Struttura modulare**: Le responsabilità dei controller sono chiare e ben definite.
- **Middleware flessibili**: È possibile implementare logica comune globale, come l'autenticazione.

#### Debolezze:
- **Centralizzazione eccessiva**: Con funzioni complesse, i Controller possono diventare difficili da gestire.
- **Aumentato rischio di coupling**: La dipendenza dal Controller per tutte le operazioni può introdurre colli di bottiglia.

---

## Perché MVC è Adatto a GitCrab

1. **Separazione delle Responsabilità**  
   MVC consente una chiara suddivisione tra gestione dei dati, business logic e interfaccia utente, semplificando manutenzione ed estensione del progetto.

2. **Modularità e Riutilizzo**  
   I modelli (`Model`) e i servizi (`Controller`) possono essere riutilizzati per molteplici scopi. Ad esempio, il `SshKeyService` è usato sia negli endpoint API che per manipolare il filesystem.

3. **Gestione delle API e delle Visualizzazioni**  
   Grazie all'impiego di uno sviluppo parallelo delle API REST e dell'interfaccia HTML, GitCrab evita duplicati di logica applicativa e garantisce una maggiore efficienza.

4. **Scalabilità**  
   La struttura modulare semplifica l'aggiunta di nuove funzionalità o l'estensione del progetto a team più grandi o ambienti di carico superiore.

---

## Vantaggi del Pattern MVC

- **Organizzazione chiara del codice**: Ogni componente è indipendente e con uno scopo ben definito.
- **Facilità di debugging**: Problemi e bug possono essere identificati all'interno del rispettivo componente.
- **Scalabilità orizzontale**: La divisione del codice rende più semplice l'implementazione di replica e bilanciamento del carico.
- **Standard di settore**: Avendo una sintassi comune, il pattern è supportato dalla maggior parte dei framework.

---

## Limiti del Pattern MVC

- **Overhead architetturale**: Per applicazioni semplici, l'adozione di MVC può essere eccessiva.
- **Proliferazione del codice**: Un uso rigoroso del pattern implica divisioni in più file e maggiore complessità.
- **Collo di bottiglia nei Controller**: La centralizzazione delle operazioni può aumentare la complessità di questa componente.

---

## Conclusione

L'adozione del pattern MVC in GitCrab si è dimostrata una scelta strategica per mantenere il progetto modulare, scalabile ed estendibile. Nonostante richieda complessità aggiuntive, permette di gestire un'applicazione in evoluzione come GitCrab con maggiore efficienza e professionalità. Questo approccio offre solidità a lungo termine e rende il progetto facilmente adattabile alle esigenze degli utenti.

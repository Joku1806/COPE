%% ccs-thesis.cls
%% Copyright (C) 2009-2018 Christoph Sommer <christoph.sommer@uibk.ac.at>
%%                         Falko Dressler <falko.dressler@uibk.ac.at>

%
% License (pick one or both):
% - Creative Commons Attribution-Share Alike 3.0 (or later)
% - GPL v2 (or later)
%

\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{ccs-thesis}[2021/02/08 The CCS Thesis Class]  % YYYY/MM/DD


% -----------------------------------------------------------------------------
% Class options
% -----------------------------------------------------------------------------
\RequirePackage{xifthen}

% [germanthesis] - Thesis is written in German
\newboolean{germanthesis}
\setboolean{germanthesis}{false}
\DeclareOption{germanthesis}{\setboolean{germanthesis}{true}}

% [plainunnumbered] - Don't print numbers on plain pages
\newboolean{plainunnumbered}
\setboolean{plainunnumbered}{false}
\DeclareOption{plainunnumbered}{\setboolean{plainunnumbered}{true}}

% [earlydraft] - Settings for quick draft printouts
\newboolean{earlydraft}
\setboolean{earlydraft}{false}
\DeclareOption{earlydraft}{\setboolean{earlydraft}{true}}

% [watermark] - Print current time/date at bottom of each page
\newboolean{watermark}
\setboolean{watermark}{false}
\DeclareOption{watermark}{\setboolean{watermark}{true}}

% [phdthesis] - switch to PhD thesis style
\newboolean{phdthesis}
\setboolean{phdthesis}{false}
\DeclareOption{phdthesis}{\setboolean{phdthesis}{true}}

% [twoside] - double sided
\newboolean{oneside}
\setboolean{oneside}{true}
\DeclareOption{twoside}{\setboolean{oneside}{false}}

% [cutmargins] - text body fills complete page
\newboolean{cutmargins}
\setboolean{cutmargins}{false}
\DeclareOption{cutmargins}{\setboolean{cutmargins}{true}}

% [nocoverpage] - no cover page when generating the pdf
\newboolean{nocoverpage}
\setboolean{nocoverpage}{false}
\DeclareOption{nocoverpage}{\setboolean{nocoverpage}{true}}

% [dedication] - dedication page after the title page
\newboolean{dedication}
\setboolean{dedication}{false}
\DeclareOption{dedication}{\setboolean{dedication}{true}}

\ProcessOptions \relax
% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% Basics
% -----------------------------------------------------------------------------
% This class is based on report
\ifthenelse{\boolean{oneside}}{
\LoadClass[a4paper,10pt,onecolumn,oneside,openright]{report}
}{
\LoadClass[a4paper,10pt,onecolumn,twoside,openright]{report}
}
\ifthenelse{\boolean{cutmargins}}{
\RequirePackage[paperwidth=365pt,paperheight=618pt,textwidth=345pt,textheight=598pt,bindingoffset=0cm,marginparwidth=3.75cm,hmarginratio=1:1,vmarginratio=1:1]{geometry}
}{
\RequirePackage[a4paper,textwidth=345pt,textheight=598pt,bindingoffset=0cm,marginparwidth=3.75cm,hmarginratio=1:1,vmarginratio=1:1]{geometry}
}

% 150 percent line spacing
\RequirePackage{setspace}
\onehalfspacing

% Allow more (and larger) floats on text pages
\renewcommand{\topfraction}{0.9}  % use up to ..% of space on top
\renewcommand{\bottomfraction}{0.8}  % use up to ..% of space on bottom
\setcounter{topnumber}{4} % place up to .. on top
\setcounter{bottomnumber}{4} % place up to .. on bottom
\setcounter{totalnumber}{4} % place up to .. total
\renewcommand{\textfraction}{0.07} % allow down to ..% of text
\renewcommand{\floatpagefraction}{0.7} % fill at least ..% of float pages (must be less than \topfraction)

% Avoid widows and orphans (single line on bottom or top of page, respectively) at almost any cost
\clubpenalty = 10000
\widowpenalty = 10000
%\raggedbottom

% Default fonts
\newcommand{\mypageheadfont}{\normalfont}
\newcommand{\myheadingfont}{\normalfont}
%\RequirePackage[osf]{libertine}  % Linux Libertine
%\renewcommand{\mypageheadfont}{\normalfont\libertine}
%\renewcommand{\myheadingfont}{\normalfont\libertine}
\RequirePackage{lmodern}  % TT: Latin Modern
\RequirePackage{helvet}  % SF: Helvetica 
\RequirePackage[bitstream-charter,sfscaled=false]{mathdesign}  % RM: Bitstream Charter
% TODO: nicer math font?

% More encoding and typesetting fixes and tweaks
\RequirePackage[utf8]{inputenc}
\RequirePackage[T1]{fontenc}
\RequirePackage{textcomp}
\ifthenelse{\boolean{earlydraft}}{
	\newcommand{\mydisableprotrusion}{}
}{
	\RequirePackage[babel,protrusion=true,expansion=true,tracking=false,kerning=true,spacing=true]{microtype}
	\newcommand{\mydisableprotrusion}{\microtypesetup{protrusion=false}}
}
\RequirePackage{hyphenat}

% Graphics insertion and filetypes' order of precedence
\RequirePackage[pdftex]{graphicx}
\pdfminorversion=7
\DeclareGraphicsRule{.out.png}{png}{.out.png}{}
\ifthenelse{\boolean{earlydraft}}{
	\DeclareGraphicsExtensions{.thumb.jpg,.out.png,.pdf,.png,.jpg}
	\DeclareGraphicsRule{.thumb.jpg}{jpg}{.thumb.jpg}{}
}{
	\DeclareGraphicsExtensions{.out.png,.pdf,.png,.jpg}
}

% Typesetting URLs
\RequirePackage[lowtilde,hyphens]{url}
\urlstyle{tt}
\g@addto@macro\UrlSpecials{%
	% re-define character '!' to insert a hard line break
	\do\!{\raisebox{0ex}[0ex][0ex]{\color{gray}\ensuremath{\searrow}}\newline}%
}

% For hyperref<->algorithm interaction
\RequirePackage{float}

% Cross-references are clickable
\RequirePackage[pdftex,bookmarksnumbered=true,colorlinks=false,pdfdisplaydoctitle=true,breaklinks=true,bookmarks=true,bookmarksopenlevel=0]{hyperref}

% Captions
\RequirePackage[font=small, labelfont=bf, labelsep=endash, margin=1cm]{caption}

% Subfigures
\RequirePackage[margin=0cm]{subfig}

% Citations
\RequirePackage{csquotes}
\RequirePackage[style=ieee,doi=true,isbn=false,url=false,mincitenames=1,maxcitenames=3,minbibnames=10,maxbibnames=10]{biblatex}
\DeclareFieldFormat{title}{#1} % never apply title casing, even if bibtex field is unprotected
\DeclareFieldFormat{sentencecase}{#1} % never apply sentence casing, even if bibtex field is unprotected
\RequirePackage{xpatch}
\xpatchbibmacro{textcite}{\addspace}{\addnbspace}{}{}
\xpatchbibmacro{Textcite}{\addspace}{\addnbspace}{}{}
\DefineBibliographyStrings{english}{
	andothers = et~al\adddot\addspace
}
\defbibheading{bibliography}[\bibname]{%
	\chapter*{#1}%
	\addcontentsline{toc}{chapter}{#1}%
	\chaptermark{#1}%
}
\bibliography{references}

% The "SI" and "num" commands for typesetting units
\RequirePackage[binary-units,range-phrase={--},per-mode=symbol,detect-all,forbid-literal-units,range-units=single]{siunitx}
\ifthenelse{\boolean{germanthesis}}{
	\sisetup{locale=DE}
}{
}
\DeclareSIUnit\byte{Byte}
\DeclareSIUnit\decibelm{dBm}
\DeclareSIUnit\mph{mph}
% siunitx v3 removed the `s` column type.
% From the "Upgrading from version 2" section of the manual:
\RequirePackage{collcell}
\newcolumntype{s}{>{\collectcell\unit}c<{\endcollectcell}}

% Commands for beautiful tables
\RequirePackage{booktabs}

% Equation typesetting fixes and tweaks + proof/theorem environment
\RequirePackage{amsmath}
\RequirePackage{amsthm}
\newtheorem{thm}{Theorem}[section]

% Allow list iteration over comma separated parameters
\RequirePackage{pgf}
\RequirePackage{pgffor}

% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% Internationalization support
% -----------------------------------------------------------------------------

% Multi-language support using Babel
\ifthenelse{\boolean{germanthesis}}{
	\RequirePackage[shorthands=off,american,ngerman]{babel}
}{
	\RequirePackage[shorthands=off,ngerman,american]{babel}
}

% % Don't indent paragraphs in german theses. Instead, leave blank space between paragraphs.
% \ifthenelse{\boolean{germanthesis}}{
% 	\RequirePackage{parskip}
% }{
% }

% Time and date functions
\RequirePackage{datetime}
\newdateformat{germandateformat}{%
	\THEDAY{}. \monthnamengerman[\THEMONTH]{} \THEYEAR{}%
}
\newdateformat{americandateformat}{%
	\THEDAY{} \monthnameamerican[\THEMONTH]{} \THEYEAR{}%
}

% Clever References
\ifthenelse{\boolean{germanthesis}}{
	\RequirePackage[ngerman]{cleveref}[2009/17/04]
	\def\cref{\Cref}
	\def\crefrange{\Crefrange}
	\crefname{lstlisting}{Listing}{Listings}
	\Crefname{lstlisting}{Listing}{Listings}
	\crefname{listing}{Listing}{Listings}
	\Crefname{listing}{Listing}{Listings}
	\crefname{algorithm}{Algorithmus}{Algorithmen}
	\Crefname{algorithm}{Algorithmus}{Algorithmen}
}{
	\RequirePackage{cleveref}[2009/17/04]
	\def\cref{\Cref}
	\def\crefrange{\Crefrange}
	\crefname{lstlisting}{listing}{listings}
	\Crefname{lstlisting}{Listing}{Listings}
	\crefname{listing}{listing}{listings}
	\Crefname{listing}{Listing}{Listings}
	\crefname{algorithm}{algorithm}{algorithms}
	\Crefname{algorithm}{Algorithm}{Algorithms}
}

% Some more language-dependent commands
\ifthenelse{\boolean{germanthesis}}{
	\newcommand{\glossarytitlename}{Abkürzungsverzeichnis}
}{
	\newcommand{\glossarytitlename}{List of Abbreviations}
}

% Language-independent timestamp
\newcommand{\timestamp}{\renewcommand{\dateseparator}{-}\yyyymmdddate\today\quad\settimeformat{hhmmsstime}\currenttime}

% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% All the simple stuff goes here
% -----------------------------------------------------------------------------

% Show overfull boxes
\overfullrule 5pt

% The Enumerate package
\RequirePackage{enumitem}

% The Acronym package
\RequirePackage{acro}

\@ifpackagelater{acro}{2020/04/29}{
\NewDocumentCommand\acrodef{mO{#1}mG{}}{\DeclareAcronym{#1}{short={#2}, long={#3}, #4}}
% Set up Acronyms for Enumerate
\newlist{acronyms}{description}{1}
\setlist[acronyms]{
	labelwidth=7em,
	leftmargin=7.5em,
	noitemsep,
	itemindent=0pt,
}
\NewAcroTemplate[list]{acronyms}{%
  \acroheading
  \acropreamble
  \begin{acronyms}
    \acronymsmapF{%
      \item[\acrowrite{short}\acroifT{alt}{/\acrowrite{alt}}]
        \acrowrite{list}%
        \acroifT{foreign}{ (\acrowrite{foreign})}%
        \acroifT{extra}{, \acrowrite{extra}}%
        \acropagefill
        \acropages
          {\acrotranslate{page}\nobreakspace}
          {\acrotranslate{pages}\nobreakspace}%
      }
      {\item\AcroRerun}
  \end{acronyms}
}
\acsetup{list/heading={chapter*},list/name={\glossarytitlename},list/template=acronyms,format/extra={\emph}}
}{
\@ifpackagelater{acro}{2019/02/17}{
	% work around bug in 2019/02/17
	\NewDocumentCommand\acrodef{mO{#1}mG{}}{\DeclareAcronym{#1}{short={#2}, long={#3}, foreign-plural={}, #4}}
}{
	\NewDocumentCommand\acrodef{mO{#1}mG{}}{\DeclareAcronym{#1}{short={#2}, long={#3}, #4}}
}
% Set up Acronyms for Enumerate
\newlist{acronyms}{description}{1}
\setlist[acronyms]{
	labelwidth=7em,
	leftmargin=7.5em,
	noitemsep,
	itemindent=0pt,
}
\DeclareAcroListStyle{acronyms}{list}{list=acronyms}
\acsetup{list-heading={chapter*},list-name={\glossarytitlename},list-style=acronyms,extra-style=comma,extra-format={\emph}}
}

% Make automatically inserted "blank pages" really blank
\RequirePackage{emptypage}

% Fix spacing around "..."
\RequirePackage{ellipsis}

% TeX ist kein Zeichenprogramm
\RequirePackage{tikz}[2010/10/13]
\usetikzlibrary{calc}

% Ensure that no floats are typeset before they are referenced
\RequirePackage{flafter}

% Provide the \FloatBarrier command that flushes all pending floats
\RequirePackage{placeins}

% Nag about deprecated LaTeX packages and commands
\RequirePackage[l2tabu, orthodox]{nag}

% Define command that will return current filename
\RequirePackage{currfile}

% Do not reset footnote counter on new chapter
\RequirePackage[]{chngcntr}
\counterwithout{footnote}{chapter}

% "Draft" watermark
\newcommand{\thewatermark}{\ifthenelse{\boolean{watermark}}{%
	\begin{tikzpicture}[remember picture, overlay]
		\node[anchor=south, outer sep=3.75cm] at (current page.south) {
			\color{gray}\currfilename\quad\timestamp
		};
	\end{tikzpicture}%
}{}}

% The "algorithm" environment
\RequirePackage{algorithmic}[2009/08/24]
\RequirePackage[plain,chapter]{algorithm}
\floatplacement{algorithm}{bp}

% The "syntax" environment
\RequirePackage[nounderscore,rounded]{syntax}

\renewcommand\fs@plain{%
\def\@fs@cfont{\rmfamily}%
\let\@fs@capt\floatc@plain%
\def\@fs@pre{\hrule\relax\kern2pt}%
\def\@fs@post{}%
\def\@fs@mid{\kern2pt\hrule\relax\vspace\abovecaptionskip\relax}%
\let\@fs@iftopcapt\iffalse%
}
\floatstyle{plain}
\restylefloat{algorithm}
\ifthenelse{\boolean{germanthesis}}{
	\floatname{algorithm}{Algorithmus}
}{
}

% Chapter/section headings and page styles
%\RequirePackage[pagestyles,nobottomtitles*]{titlesec}
\RequirePackage[pagestyles]{titlesec}

% Work around bug in titlesec version 2.10.1
\@ifpackagelater{titlesec}{2016/03/21}{}{\@ifpackagelater{titlesec}{2016/03/15}{%
	\def\ttlh@hang#1#2#3#4#5#6#7#8{%
	\gdef\ttl@makeline##1{\ttl@calc\hspace{#6}##1\ttl@calc\hspace{#7}}%
	\setlength\leftskip{#6}%
	\setlength\rightskip{#7}%
	\interlinepenalty\@M
	\ttl@changecentercr
	\ttl@beginlongest
	#1{\ifhmode\ttl@hmode@error\fi
		 \ttl@glcmds
		 \parindent\z@
		 \begingroup
			 \ifttl@label
					\noindent
					\sbox\z@{#2\strut\ttl@calc\hspace{#3}}%
					\hangindent\wd\z@
					\box\z@
			 \fi
			 #4{#8}%
			 \kern\z@\strut\@@par
		 \endgroup
		 \nobreak\ttl@midlongest#5\@@par}%
	\ttl@endlongest}
}{}}

\ifthenelse{\boolean{plainunnumbered}}{
\renewpagestyle{plain}{%
	\sethead{}{}{}
	\setfoot{}{}{\thewatermark}
}
}{
\renewpagestyle{plain}{%
	\sethead{}{}{}
	\setfoot{}{\mypageheadfont\thepage}{\thewatermark}
}
}

% Pagestyle "fancy"
\newpagestyle{fancy}{%
\sethead*{
	\bfseries
	\toptitlemarks
	\mypageheadfont\bfseries
	\ifthechapter
		{
			\ifthesection
			{\firsttitlemarks\thesection~\sectiontitle}
			{\thechapter~\chaptertitle}
		}
		{\chaptertitle}
}
{}{\mypageheadfont\bfseries\thepage}
	\setfoot{}{}{\thewatermark}
	\headrule
}
\pagestyle{fancy}

% rename (sub)paragraph to (sub)subsubsubsection
\let\myparagraph\paragraph
\renewcommand{\paragraph}{\FIXME{in order to avoid confusion with hyperref, \\paragraph has been renamed to \\subsubsubsection}}
\let\subsubsubsection\myparagraph
\let\mysubparagraph\subparagraph
\renewcommand{\subparagraph}{\FIXME{in order to avoid confusion with hyperref, \\subparagraph has been renamed to \\subsubsubsubsection}}
\let\subsubsubsubsection\mysubparagraph

% heading styles
\titleformat{\chapter}[display]{\setlength\parskip{0pt}\myheadingfont\huge\bfseries\titlerule\vspace{0.25cm}}{\LARGE\chaptertitlename~\thechapter}{1em}{}[\vspace{0.25cm}\titlerule]
\titleformat{\section}{\myheadingfont\Large\bfseries}{\thesection}{1em}{}
\titleformat{\subsection}{\myheadingfont\large\bfseries}{\thesubsection}{1em}{}
\titleformat{\subsubsection}{\myheadingfont\normalsize\bfseries}{\thesubsubsection}{1em}{}
\titleformat{\paragraph}{\myheadingfont\normalsize\bfseries}{\theparagraph}{1em}{}
\titleformat{\subparagraph}{\myheadingfont\normalsize}{\thesubparagraph}{1em}{\labelitemi~}
\titlespacing*{\chapter}{0pt}{50pt}{40pt minus \parskip}
\titlespacing*{\section}{0pt}{3.5ex plus 1ex minus .2ex - \parskip}{2.3ex plus .2ex - \parskip}
\titlespacing*{\subsection}{0pt}{3.25ex plus 1ex minus .2ex - \parskip}{1.5ex plus .2ex - \parskip}
\titlespacing*{\subsubsection}{0pt}{3.25ex plus 1ex minus .2ex - \parskip}{1.5ex plus .2ex - \parskip} 
\titlespacing*{\paragraph}{0pt}{3.25ex plus 1ex minus .2ex - \parskip}{1.5ex plus .2ex - \parskip} 
\titlespacing*{\subparagraph}{0pt}{3.25ex plus 1ex minus .2ex - \parskip}{1.5ex plus .2ex - \parskip} 
\ifthenelse{\boolean{earlydraft}}{
	\setcounter{tocdepth}{3}
	\setcounter{secnumdepth}{3}
}{
	\setcounter{tocdepth}{3}
	\setcounter{secnumdepth}{3}
}
% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% Commands for setting author, title, ...
% -----------------------------------------------------------------------------
\newcommand{\group}[1]{\gdef\thegroup{#1}}
\renewcommand{\author}[1]{\gdef\theauthor{#1}}
\renewcommand{\title}[1]{\gdef\thetitle{#1}}
\newcommand{\thesistype}[1]{\gdef\thethesistype{#1}}
\newcommand{\thesiscite}[1]{\gdef\thethesiscite{#1}}
\newcommand{\thesisinstitution}[1]{\gdef\thethesisinstitution{#1}}
\newcommand{\thesisschool}[1]{\gdef\thethesisschool{#1}}
\newcommand{\thesiscity}[1]{\gdef\thethesiscity{#1}}
\newcommand{\birthday}[1]{\gdef\thebirthday{#1}}
\newcommand{\birthplace}[1]{\gdef\thebirthplace{#1}}
\newcommand{\studentid}[1]{\gdef\thestudentid{#1}}
\newcommand{\advisors}[1]{\gdef\theadvisors{#1}}
\newcommand{\referees}[1]{\gdef\thereferees{#1}}
\newcommand{\refereechair}[1]{\gdef\therefereechair{#1}}
\newcommand{\thesubmissiondate}[1]{\gdef\thesubmissiondate{#1}}
\newcommand{\thedefensedate}[1]{\gdef\thedefensedate{#1}}
\newcommand{\orcid}[1]{\gdef\theorcid{#1}}
\newcommand{\faculty}[1]{\gdef\thefaculty{#1}}
\newcommand{\university}[1]{\gdef\theuniversity{#1}}
\newcommand{\placeandyear}[1]{\gdef\theplaceandyear{#1}}
% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% Listings
% -----------------------------------------------------------------------------
\RequirePackage[savemem]{listings}
\ifthenelse{\boolean{germanthesis}}{
	\renewcommand{\lstlistlistingname}{Listingverzeichnis}
}{
	\renewcommand{\lstlistlistingname}{List of Listings}
}
\lstset{
	language=C,
	frame=lines,
	framesep=5pt,
	captionpos=b,
	abovecaptionskip=1em,
	numbers=left,
	xleftmargin=15pt,
	framexleftmargin=15pt,
	numberstyle=\tiny,
	numbersep=5pt,
	stepnumber=1,
	fontadjust,
	tabsize=2,
	showtabs=false,
	showspaces=false,
	showstringspaces=false,
	breaklines=true,
	breakatwhitespace=true,
	keywordstyle=\color{black},
	identifierstyle=\bfseries\color{blue},
	commentstyle=\color{red},
	prebreak=\raisebox{0ex}[0ex][0ex]{\color{gray}\ensuremath{\searrow}},
	stringstyle=\bfseries\color{blue},
	basicstyle=\ttfamily\bfseries\color{black}\footnotesize,
}
\lstdefinestyle{txt}{
	float,
	floatplacement=bp,
	language=C
}
\lstdefinestyle{xml}{
	float,
	floatplacement=bp,
	language=XML
}
% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% TOCs
% -----------------------------------------------------------------------------

% Include list-of-Xs in table of contents
\ifthenelse{\boolean{earlydraft}}{
	\RequirePackage[]{tocbibind}
	\renewcommand{\lstlistoflistings}{\begingroup\tocfile{\lstlistlistingname}{lol}\endgroup}
	\newcommand{\listofabbreviations}{\addcontentsline{toc}{chapter}{\glossarytitlename}\printacronyms}
	\newcommand{\mymaketodotocentry}{\todototoc}
}{
	\RequirePackage[nottoc,notlot,notlof]{tocbibind}
	\newcommand{\listofabbreviations}{\printacronyms}
	\newcommand{\mymaketodotocentry}{}
}

% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% Visible TODO and FIXME markers
% -----------------------------------------------------------------------------
\ifthenelse{\boolean{germanthesis}}{
	\RequirePackage[colorinlistoftodos, textwidth=3.75cm, shadow, ngerman]{todonotes}
	\@todonotes@SetTodoListName{Offene Punkte}
}{
	\RequirePackage[colorinlistoftodos, textwidth=3.75cm, shadow]{todonotes}
}
\newcounter{TODOCOUNT}
\newcommand{\TODO}[1]{\vspace{0.5em}\todo[inline, color=orange]{#1}\stepcounter{TODOCOUNT}}
\newcommand{\FIXME}[1]{\todo[size=\small, color=red]{#1}\stepcounter{TODOCOUNT}}
\AtEndDocument{
	\ifnum\value{TODOCOUNT}>0
		%\cleardoublepage
		\mymaketodotocentry\listoftodos
	\fi
}
% -----------------------------------------------------------------------------


% -----------------------------------------------------------------------------
% Title page
% -----------------------------------------------------------------------------
\newcommand{\coverpagetextsmall}[1]{%
	\@ccsgroupcoverpage@smalltextsize%
	#1%
	\vspace{0.5cm}\\%
}

\newcommand{\coverpagetextlarge}[1]{%
	\vspace{0.25cm}%
	\@ccsgroupcoverpage@largetextsize%
	#1%
	\@ccsgroupcoverpage@smalltextsize%
	\vspace{0.5cm}\\%
}
\renewcommand{\maketitle}{%

% Do some stuff we had to postpone until after e.g. \thetitle was set
\hypersetup{%
  pdftitle = {\thetitle},
  pdfsubject = {\thethesistype},
  pdfauthor = {\theauthor},
  pdfborder=0 0 0
}

% Count authors
\newcounter{@ccsauthorcount}%
\foreach \n [count=\ni] in \theauthor {%
	\stepcounter{@ccsauthorcount}%
}%

\pagenumbering{alph}

% Output cover page
\ifthenelse{\boolean{cutmargins} \OR \boolean{nocoverpage}}{
	% no cover page if cutmargins or nocoverpage is set
}
{%

\tikzoption{text @ccsgroupcoverpage@ badly raggedleft}[]{\def\tikz@text@action{\raggedleft\relax}}

\begingroup
\pagestyle{empty}
\setcounter{page}\@ne

\def\@ccsgroupcoverpage@titlesize{\@setfontsize{\@ccsgroupcoverpage@titlesize}{20}{24pt}}
\def\@ccsgroupcoverpage@subtitlesize{\@setfontsize{\@ccsgroupcoverpage@subtitlesize}{14}{17pt}}
\def\@ccsgroupcoverpage@citeassize{\@setfontsize{\@ccsgroupcoverpage@citeassize}{8}{9pt}}
\def\@ccsgroupcoverpage@addressize{\@setfontsize{\@ccsgroupcoverpage@addressize}{8}{9pt}}
\def\@ccsgroupcoverpage@smalltextsize{\@setfontsize{\@ccsgroupcoverpage@smalltextsize}{14}{17pt}}
\def\@ccsgroupcoverpage@largetextsize{\@setfontsize{\@ccsgroupcoverpage@largetextsize}{20}{24pt}}

\renewcommand{\baselinestretch}{1.00}
\definecolor{TURed}{RGB}{197,14,31}
\begin{tikzpicture}[remember picture,overlay] \node[xshift=0cm,yshift=0cm] at (current page.south west) { \begin{tikzpicture}[remember picture, overlay]

	%\pgftext[bottom,left,at={\pgfpoint{0cm}{0cm}}] {%
		%\includegraphics{ccs/cover-bg}%
	%}

    \coordinate (O) at (current page.south west);
    \node[inner sep=0pt, anchor=north east] (logo) at ([xshift=-15mm, yshift=-15mm]current page.north east)
    {\includegraphics[]{tkn/TU_Logo_lang_4c_rot.pdf}};

    \draw[fill=TURed,line width=0pt,rounded corners=0.5mm] let \p1 = (logo.south east) in ([yshift=-15mm]logo.south east) rectangle ([yshift=-16mm]15mm,\y1);

	%%% multi-line text
	\pgftext[top,left,at={\pgfpoint{2.5cm}{20cm}}] {%
		\parbox{15cm}{%
			\raggedright%
			\sffamily%
			\@ccsgroupcoverpage@largetextsize%
			\renewcommand{\baselinestretch}{1.25}%
			\coverpagetextsmall{\nohyphens{%
				\foreach \n [count=\ni] in \theauthor {%
					\ifnum\ni=1%
					\else
						\ifnum\ni=\the@ccsauthorcount%
							~und~%
						\else%
							,~%
						\fi%
					\fi%
					\n
				}%
			}}%
			\coverpagetextlarge{\nohyphens{\thetitle}}%
			\coverpagetextsmall{\thethesistype}%
			\coverpagetextsmall{\americandateformat\today}%
		}%
	};
	\draw[xshift=2.5cm,yshift=6cm] node [above right,text width=15cm, text badly ragged] {%
		\sffamily%
		\@ccsgroupcoverpage@citeassize%
		\renewcommand{\baselinestretch}{1.00}%
		\begin{otherlanguage*}{american}%
			Please cite as:\\%
			\nohyphens{%
				\foreach \n [count=\ni] in \theauthor {%
					\ifnum\ni=1%
					\else
						\ifnum\ni=\the@ccsauthorcount%
							~and~%
						\else%
							,~%
						\fi%
					\fi%
					\n
				}%
			, ``\thetitle,'' \thethesiscite, \thethesisinstitution, \thethesisschool, \thethesiscity, \monthname~\the\year.}%
		\end{otherlanguage*}
	};
    
    \node[anchor=south east] (footer) at ([xshift=-15mm, yshift=15mm]current page.south east)
    {%
        \parbox{12cm}{
        \raggedleft
		\sffamily%
		\@ccsgroupcoverpage@addressize%
		\renewcommand{\baselinestretch}{1.00}%
		Fachgebiet Telekommunikationsnetze\\%
		Technische Universität Berlin, Germany\\[1\baselineskip]%
		Einsteinufer 25 $\cdot$ 10587 Berlin $\cdot$ Germany\\[1\baselineskip]%
		\href{https://www.tkn.tu-berlin.de/}{https://www.tkn.tu-berlin.de/}%
    }
	};

    \path let \p1=(footer.north) in node[anchor=north west, inner sep=0pt] (tknlogo) at (15mm,\y1)
    {%
        \includegraphics[width=7cm]{tkn/tkn-logo-alternative.pdf}
	};

    \draw[fill=TURed,line width=0pt,rounded corners=0.5mm] let \p1 = (footer.north east) in ([yshift=5mm]footer.north east) rectangle ([yshift=6mm]15mm,\y1);
\end{tikzpicture} }; 
\end{tikzpicture}

\cleardoublepage
\setcounter{page}\@ne
\endgroup
}

% Title pages use single line spacing, no parindent and no parskips
\begin{singlespace}
\setlength\parindent{0pt} 
\setlength\parskip{0pt} 

%
% Now output the title page
%

\pagestyle{empty}
\pagenumbering{roman}

\ifthenelse{\boolean{phdthesis}}
{
\thispagestyle{empty}

% PhD Thesis Title Page
% https://www.static.tu.berlin/fileadmin/www/10002444/Dokumente/Forschen_Publizieren/Dissertationsstelle/UV_diss_mustertitelblatt_dt.pdf

\vspace*{1.0cm}

\begin{center}

{\LARGE \textbf{\thetitle}}\\[6\baselineskip]

{\large
vorgelegt von\\

\foreach \n [count=\ni] in \theauthor {%
    M. Sc.\\
    \n%
    \ifnum\ni=\the@ccsauthorcount%
        \ifnum\the@ccsauthorcount=1%
            \\%
            ORCID: \theorcid \\[3\baselineskip]%
%            geb. am \thebirthday\\
%            in \thebirthplace\\[2\baselineskip]%
        \else%
            \\[3\baselineskip]%
        \fi%
    \else%
        \\%
    \fi%
}%

an der \thefaculty\\
der \theuniversity\\
zur Erlangung des akademischen Grades\\[\baselineskip]

Doktor der Ingenieurwissenschaften\\
-- Dr.-Ing. --\\[\baselineskip]

genehmigte \thethesistype

}~\\[3\baselineskip]

%\uppercase{Doktor der Ingenieurwissenschaften}\\
%D\,O\,K\,T\,O\,R\,~\,D\,E\,R\,~\,N\,A\,T\,U\,R\,W\,I\,S\,S\,E\,N\,S\,C\,H\,A\,F\,T\,E\,N\\
%D\,O\,K\,T\,O\,R\,~\,D\,E\,R\,~\,I\,N\,G\,E\,N\,I\,E\,U\,R\,W\,I\,S\,S\,E\,N\,S\,C\,H\,A\,F\,T\,E\,N\\

\end{center}

{\large
Promotionsausschuss:\\

Vorsitzender: \therefereechair\\
\foreach \n [count=\ni] in \thereferees {%
    \ifnum\ni=1%
    \else%
        \newline
    \fi%
    Gutachter: \n%
	}%
\\

%\begin{tabular}{>{\raggedleft}p{5cm}p{6cm}}
%    Vorsitzender: & \textbf{\therefereechair}\\
%\ifdef\theadvisors{
%Betreuer:   & %
%    \foreach \n [count=\ni] in \theadvisors {%
%        \ifnum\ni=1%
%        \else%
%            \newline
%        \fi%
%        \textbf{\n}%
%    }%
%\\[.2\baselineskip]
%}{}
%Gutachter:  & %
%    \foreach \n [count=\ni] in \thereferees {%
%        \ifnum\ni=1%
%        \else%
%            \newline
%        \fi%
%        \textbf{\n}%
%    }%
%\\
%~ & ~\\
%Tag der Abgabe: & \textbf{\thesubmissiondate}\\[.2\baselineskip]
%Tag der Promotion: & \textbf{\thedefensedate}\\
%\end{tabular}

Tag der wissenschaftlichen Aussprache: \thedefensedate

\begin{center}
\vspace*{0.7cm}
\theplaceandyear
\end{center}

}

\cleardoublepage



}
% else: Master Thesis title page
{
\thispagestyle{empty}

\vspace*{0.8cm}

\begin{center}

{\LARGE \textbf{\thetitle}}\\[2\baselineskip]

{\large
\thethesistype\\[2\baselineskip]

vorgelegt von\\[2\baselineskip]

\foreach \n [count=\ni] in \theauthor {%
	\textbf{\n}%
	\ifnum\ni=\the@ccsauthorcount%
		\ifnum\the@ccsauthorcount=1%
			\\[1\baselineskip]%
			geb. am \thebirthday\\
			in \thebirthplace\\
			Matrikelnummer: \thestudentid\\
			[2\baselineskip]%
		\else%
			\\[2\baselineskip]%
		\fi%
	\else%
		\\%
	\fi%
}%

angefertigt am Fachgebiet \\[2\baselineskip]

\textbf{%
Telekommunikationsnetze\\[1.2\baselineskip]
Fakultät für Elektrotechnik und Informatik\\[0.2\baselineskip]
Technische Universität Berlin\\[0.2\baselineskip]
}
}~\\[1\baselineskip]

\begin{tabular}{>{\raggedleft}p{5cm}p{6cm}}
\ifdef\theadvisors{%
Betreuer:	& %
	\foreach \n [count=\ni] in \theadvisors {%
		\ifnum\ni=1%
		\else%
			\newline
		\fi%
		\textbf{\n}%
	}%
\\
}{}
Gutachter:	& %
	\foreach \n [count=\ni] in \thereferees {%
		\ifnum\ni=1%
		\else%
			\newline
		\fi%
		\textbf{\n}%
	}%
\\
~ & ~\\
Abgabe der Arbeit: & \textbf{\germandateformat\today}\\
\end{tabular}

\end{center}

\clearpage
}
\end{singlespace}

%
% The delaration comes here
%

\ifthenelse{\boolean{phdthesis}}{}
{
\thispagestyle{empty}

\begin{otherlanguage*}{ngerman}
{\Large\noindent%
Erklärung \newline}

\noindent%
Ich versichere, dass ich die Arbeit ohne fremde Hilfe und ohne Benutzung anderer als der angegebenen Quellen angefertigt habe und dass die Arbeit in gleicher oder ähnlicher Form noch keiner anderen Prüfungsbehörde vorgelegen hat und von dieser als Teil einer Prüfungsleistung angenommen wurde.\\
Alle Ausführungen, die wörtlich oder sinngemäß übernommen wurden, sind als solche gekennzeichnet.

\end{otherlanguage*}


\vspace{2cm}


\foreach \n [count=\ni] in \theauthor {%
	\noindent%
	\vspace{1cm}\\
    \rule{5cm}{0.5pt}\\
	(\n)\\
	\ifnum\ni=\the@ccsauthorcount%
		Berlin, den \americandateformat\today
	\fi
}%



\cleardoublepage
}

\pagestyle{fancy}

% Dedication start

\ifthenelse{\boolean{dedication}}
{
	\thispagestyle{empty}
	\vspace*{5.0cm}

	{
	\Large
	\itshape
	
	\begin{flushright}
	Dedication text \\
	goes here...
	\end{flushright}
	}
	\cleardoublepage
}
	
% Dedication ends

}
% -----------------------------------------------------------------------------

\RequirePackage{subfiles}


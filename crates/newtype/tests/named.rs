use newtype::{Named, Unnamed};
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
    #[name("dimmed-low")]
    Dimmed,
}

struct Names;

impl Names {
    const LIT: &'static str = Lamp::On.name();

    fn parsed(text: &&str) -> Result<Lamp, Unnamed<Lamp>> {
        text.parse()
    }

    fn refused(text: &&str) -> Option<String> {
        text.parse::<Lamp>().err().map(|refusal| refusal.to_string())
    }
}

laws::table!(each_variant_has_its_declared_name, |lamp: &Lamp| lamp.name(), [Lamp::On => "on", Lamp::Off => "off", Lamp::Dimmed => "dimmed-low"]);

laws::table!(display_writes_the_declared_name, |lamp: &Lamp| lamp.to_string(), [Lamp::On => "on", Lamp::Dimmed => "dimmed-low"]);

laws::table!(display_honours_width_and_alignment, |lamp: &Lamp| (format!("{lamp:>5}"), format!("{lamp:<5}|"), format!("{lamp:^7}")), [
    Lamp::On => (String::from("   on"), String::from("on   |"), String::from("  on   ")),
]);

laws::table!(from_str_reads_each_name_and_refuses_any_other_text, Names::parsed, [
    "off" => Ok(Lamp::Off),
    "dimmed-low" => Ok(Lamp::Dimmed),
    "On" => Err(Unnamed::new("On")),
    "" => Err(Unnamed::new("")),
]);

laws::table!(a_refusal_names_the_text_and_every_name, Names::refused, ["dim" => Some(String::from("`dim` is not one of: on, off, dimmed-low")), "on" => None]);

laws::table!(every_variant_parses_back_from_its_name, |lamp: &Lamp| lamp.name().parse::<Lamp>(), [Lamp::On => Ok(Lamp::On), Lamp::Off => Ok(Lamp::Off), Lamp::Dimmed => Ok(Lamp::Dimmed)]);

#[test]
fn a_name_is_a_compile_time_constant() {
    assert_eq!(Names::LIT, "on");
    assert_eq!(Lamp::VARIANTS.iter().map(|lamp| lamp.name()).collect::<Vec<_>>(), ["on", "off", "dimmed-low"]);
}

#[derive(Named, VariantArray, Clone, Copy)]
enum Bare {
    #[name("bare")]
    Only,
}

impl Bare {
    fn read(text: &str) -> Result<Bare, Box<dyn std::error::Error>> {
        Ok(text.parse::<Bare>()?)
    }
}

#[test]
fn a_refusal_boxes_into_dyn_error_for_an_enum_without_debug_or_partial_eq() {
    assert_eq!(
        Bare::read("other").err().map(|error| error.to_string()),
        Some("`other` is not one of: bare".to_owned())
    );
    assert!(Bare::read("bare").is_ok());
}

#[derive(Named, VariantArray, Clone, Copy)]
#[accessor(glyph: &'static str)]
#[accessor(rank: u8)]
#[accessor(lit: bool)]
#[accessor(mark: char)]
enum Lit {
    #[name("hi")]
    #[value(glyph = "*")]
    #[value(rank = 7)]
    #[value(lit = true)]
    #[value(mark = 'h')]
    High,
    #[name("lo")]
    #[value(mark = 'l')]
    #[value(lit = false)]
    #[value(rank = 1)]
    #[value(glyph = "-")]
    Low,
}

struct Accessors;

impl Accessors {
    const GLYPH: &'static str = Lit::High.glyph();
    const RANK: u8 = Lit::Low.rank();
    const LIT: bool = Lit::High.lit();
    const MARK: char = Lit::Low.mark();
}

laws::table!(accessors_read_each_variants_values, |lit: &Lit| (lit.glyph(), lit.rank(), lit.lit(), lit.mark()), [
    Lit::High => ("*", 7, true, 'h'),
    Lit::Low => ("-", 1, false, 'l'),
]);

#[test]
fn accessors_are_compile_time_constants() {
    assert_eq!(Accessors::GLYPH, "*");
    assert_eq!(Accessors::RANK, 1);
    assert_eq!(u8::from(Accessors::LIT), 1);
    assert_eq!(Accessors::MARK, 'l');
}

macro_rules! wide {
    ($($variant:ident $name:literal)*) => {
        #[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
        enum Wide {
            $(
                #[name($name)]
                $variant,
            )*
        }
    };
}

wide! {
    V0 "v0" V1 "v1" V2 "v2" V3 "v3" V4 "v4" V5 "v5" V6 "v6" V7 "v7"
    V8 "v8" V9 "v9" V10 "v10" V11 "v11" V12 "v12" V13 "v13" V14 "v14" V15 "v15"
    V16 "v16" V17 "v17" V18 "v18" V19 "v19" V20 "v20" V21 "v21" V22 "v22" V23 "v23"
    V24 "v24" V25 "v25" V26 "v26" V27 "v27" V28 "v28" V29 "v29" V30 "v30" V31 "v31"
    V32 "v32" V33 "v33" V34 "v34" V35 "v35" V36 "v36" V37 "v37" V38 "v38" V39 "v39"
    V40 "v40" V41 "v41" V42 "v42" V43 "v43" V44 "v44" V45 "v45" V46 "v46" V47 "v47"
    V48 "v48" V49 "v49" V50 "v50" V51 "v51" V52 "v52" V53 "v53" V54 "v54" V55 "v55"
    V56 "v56" V57 "v57" V58 "v58" V59 "v59" V60 "v60" V61 "v61" V62 "v62" V63 "v63"
    V64 "v64" V65 "v65" V66 "v66" V67 "v67" V68 "v68" V69 "v69" V70 "v70" V71 "v71"
    V72 "v72" V73 "v73" V74 "v74" V75 "v75" V76 "v76" V77 "v77" V78 "v78" V79 "v79"
    V80 "v80" V81 "v81" V82 "v82" V83 "v83" V84 "v84" V85 "v85" V86 "v86" V87 "v87"
    V88 "v88" V89 "v89" V90 "v90" V91 "v91" V92 "v92" V93 "v93" V94 "v94" V95 "v95"
    V96 "v96" V97 "v97" V98 "v98" V99 "v99" V100 "v100" V101 "v101" V102 "v102" V103 "v103"
    V104 "v104" V105 "v105" V106 "v106" V107 "v107" V108 "v108" V109 "v109" V110 "v110" V111 "v111"
    V112 "v112" V113 "v113" V114 "v114" V115 "v115" V116 "v116" V117 "v117" V118 "v118" V119 "v119"
    V120 "v120" V121 "v121" V122 "v122" V123 "v123" V124 "v124" V125 "v125" V126 "v126" V127 "v127"
    V128 "v128" V129 "v129" V130 "v130" V131 "v131" V132 "v132" V133 "v133" V134 "v134" V135 "v135"
    V136 "v136" V137 "v137" V138 "v138" V139 "v139" V140 "v140" V141 "v141" V142 "v142" V143 "v143"
    V144 "v144" V145 "v145" V146 "v146" V147 "v147" V148 "v148" V149 "v149" V150 "v150" V151 "v151"
    V152 "v152" V153 "v153" V154 "v154" V155 "v155" V156 "v156" V157 "v157" V158 "v158" V159 "v159"
    V160 "v160" V161 "v161" V162 "v162" V163 "v163" V164 "v164" V165 "v165" V166 "v166" V167 "v167"
    V168 "v168" V169 "v169" V170 "v170" V171 "v171" V172 "v172" V173 "v173" V174 "v174" V175 "v175"
    V176 "v176" V177 "v177" V178 "v178" V179 "v179" V180 "v180" V181 "v181" V182 "v182" V183 "v183"
    V184 "v184" V185 "v185" V186 "v186" V187 "v187" V188 "v188" V189 "v189" V190 "v190" V191 "v191"
    V192 "v192" V193 "v193" V194 "v194" V195 "v195" V196 "v196" V197 "v197" V198 "v198" V199 "v199"
    V200 "v200" V201 "v201" V202 "v202" V203 "v203" V204 "v204" V205 "v205" V206 "v206" V207 "v207"
    V208 "v208" V209 "v209" V210 "v210" V211 "v211" V212 "v212" V213 "v213" V214 "v214" V215 "v215"
    V216 "v216" V217 "v217" V218 "v218" V219 "v219" V220 "v220" V221 "v221" V222 "v222" V223 "v223"
    V224 "v224" V225 "v225" V226 "v226" V227 "v227" V228 "v228" V229 "v229" V230 "v230" V231 "v231"
    V232 "v232" V233 "v233" V234 "v234" V235 "v235" V236 "v236" V237 "v237" V238 "v238" V239 "v239"
    V240 "v240" V241 "v241" V242 "v242" V243 "v243" V244 "v244" V245 "v245" V246 "v246" V247 "v247"
    V248 "v248" V249 "v249" V250 "v250" V251 "v251" V252 "v252" V253 "v253" V254 "v254" V255 "v255"
    V256 "v256" V257 "v257" V258 "v258" V259 "v259" V260 "v260" V261 "v261" V262 "v262" V263 "v263"
    V264 "v264" V265 "v265" V266 "v266" V267 "v267" V268 "v268" V269 "v269" V270 "v270" V271 "v271"
    V272 "v272" V273 "v273" V274 "v274" V275 "v275" V276 "v276" V277 "v277" V278 "v278" V279 "v279"
    V280 "v280" V281 "v281" V282 "v282" V283 "v283" V284 "v284" V285 "v285" V286 "v286" V287 "v287"
    V288 "v288" V289 "v289" V290 "v290" V291 "v291" V292 "v292" V293 "v293" V294 "v294" V295 "v295"
    V296 "v296" V297 "v297" V298 "v298" V299 "v299" V300 "v300" V301 "v301" V302 "v302" V303 "v303"
    V304 "v304" V305 "v305" V306 "v306" V307 "v307" V308 "v308" V309 "v309" V310 "v310" V311 "v311"
    V312 "v312" V313 "v313" V314 "v314" V315 "v315" V316 "v316" V317 "v317" V318 "v318" V319 "v319"
    V320 "v320" V321 "v321" V322 "v322" V323 "v323" V324 "v324" V325 "v325" V326 "v326" V327 "v327"
    V328 "v328" V329 "v329" V330 "v330" V331 "v331" V332 "v332" V333 "v333" V334 "v334" V335 "v335"
    V336 "v336" V337 "v337" V338 "v338" V339 "v339" V340 "v340" V341 "v341" V342 "v342" V343 "v343"
    V344 "v344" V345 "v345" V346 "v346" V347 "v347" V348 "v348" V349 "v349" V350 "v350" V351 "v351"
    V352 "v352" V353 "v353" V354 "v354" V355 "v355" V356 "v356" V357 "v357" V358 "v358" V359 "v359"
    V360 "v360" V361 "v361" V362 "v362" V363 "v363" V364 "v364" V365 "v365" V366 "v366" V367 "v367"
    V368 "v368" V369 "v369" V370 "v370" V371 "v371" V372 "v372" V373 "v373" V374 "v374" V375 "v375"
    V376 "v376" V377 "v377" V378 "v378" V379 "v379" V380 "v380" V381 "v381" V382 "v382" V383 "v383"
    V384 "v384" V385 "v385" V386 "v386" V387 "v387" V388 "v388" V389 "v389" V390 "v390" V391 "v391"
    V392 "v392" V393 "v393" V394 "v394" V395 "v395" V396 "v396" V397 "v397" V398 "v398" V399 "v399"
    V400 "v400" V401 "v401" V402 "v402" V403 "v403" V404 "v404" V405 "v405" V406 "v406" V407 "v407"
    V408 "v408" V409 "v409" V410 "v410" V411 "v411" V412 "v412" V413 "v413" V414 "v414" V415 "v415"
    V416 "v416" V417 "v417" V418 "v418" V419 "v419" V420 "v420" V421 "v421" V422 "v422" V423 "v423"
    V424 "v424" V425 "v425" V426 "v426" V427 "v427" V428 "v428" V429 "v429" V430 "v430" V431 "v431"
    V432 "v432" V433 "v433" V434 "v434" V435 "v435" V436 "v436" V437 "v437" V438 "v438" V439 "v439"
    V440 "v440" V441 "v441" V442 "v442" V443 "v443" V444 "v444" V445 "v445" V446 "v446" V447 "v447"
    V448 "v448" V449 "v449" V450 "v450" V451 "v451" V452 "v452" V453 "v453" V454 "v454" V455 "v455"
    V456 "v456" V457 "v457" V458 "v458" V459 "v459" V460 "v460" V461 "v461" V462 "v462" V463 "v463"
    V464 "v464" V465 "v465" V466 "v466" V467 "v467" V468 "v468" V469 "v469" V470 "v470" V471 "v471"
    V472 "v472" V473 "v473" V474 "v474" V475 "v475" V476 "v476" V477 "v477" V478 "v478" V479 "v479"
    V480 "v480" V481 "v481" V482 "v482" V483 "v483" V484 "v484" V485 "v485" V486 "v486" V487 "v487"
    V488 "v488" V489 "v489" V490 "v490" V491 "v491" V492 "v492" V493 "v493" V494 "v494" V495 "v495"
    V496 "v496" V497 "v497" V498 "v498" V499 "v499" V500 "v500" V501 "v501" V502 "v502" V503 "v503"
    V504 "v504" V505 "v505" V506 "v506" V507 "v507" V508 "v508" V509 "v509" V510 "v510" V511 "v511"
    V512 "v512" V513 "v513" V514 "v514" V515 "v515" V516 "v516" V517 "v517" V518 "v518" V519 "v519"
    V520 "v520" V521 "v521" V522 "v522" V523 "v523" V524 "v524" V525 "v525" V526 "v526" V527 "v527"
    V528 "v528" V529 "v529" V530 "v530" V531 "v531" V532 "v532" V533 "v533" V534 "v534" V535 "v535"
    V536 "v536" V537 "v537" V538 "v538" V539 "v539" V540 "v540" V541 "v541" V542 "v542" V543 "v543"
    V544 "v544" V545 "v545" V546 "v546" V547 "v547" V548 "v548" V549 "v549" V550 "v550" V551 "v551"
    V552 "v552" V553 "v553" V554 "v554" V555 "v555" V556 "v556" V557 "v557" V558 "v558" V559 "v559"
    V560 "v560" V561 "v561" V562 "v562" V563 "v563" V564 "v564" V565 "v565" V566 "v566" V567 "v567"
    V568 "v568" V569 "v569" V570 "v570" V571 "v571" V572 "v572" V573 "v573" V574 "v574" V575 "v575"
    V576 "v576" V577 "v577" V578 "v578" V579 "v579" V580 "v580" V581 "v581" V582 "v582" V583 "v583"
    V584 "v584" V585 "v585" V586 "v586" V587 "v587" V588 "v588" V589 "v589" V590 "v590" V591 "v591"
    V592 "v592" V593 "v593" V594 "v594" V595 "v595" V596 "v596" V597 "v597" V598 "v598" V599 "v599"
    V600 "v600" V601 "v601" V602 "v602" V603 "v603" V604 "v604" V605 "v605" V606 "v606" V607 "v607"
    V608 "v608" V609 "v609" V610 "v610" V611 "v611" V612 "v612" V613 "v613" V614 "v614" V615 "v615"
    V616 "v616" V617 "v617" V618 "v618" V619 "v619" V620 "v620" V621 "v621" V622 "v622" V623 "v623"
    V624 "v624" V625 "v625" V626 "v626" V627 "v627" V628 "v628" V629 "v629" V630 "v630" V631 "v631"
    V632 "v632" V633 "v633" V634 "v634" V635 "v635" V636 "v636" V637 "v637" V638 "v638" V639 "v639"
    V640 "v640" V641 "v641" V642 "v642" V643 "v643" V644 "v644" V645 "v645" V646 "v646" V647 "v647"
    V648 "v648" V649 "v649" V650 "v650" V651 "v651" V652 "v652" V653 "v653" V654 "v654" V655 "v655"
    V656 "v656" V657 "v657" V658 "v658" V659 "v659" V660 "v660" V661 "v661" V662 "v662" V663 "v663"
    V664 "v664" V665 "v665" V666 "v666" V667 "v667" V668 "v668" V669 "v669" V670 "v670" V671 "v671"
    V672 "v672" V673 "v673" V674 "v674" V675 "v675" V676 "v676" V677 "v677" V678 "v678" V679 "v679"
    V680 "v680" V681 "v681" V682 "v682" V683 "v683" V684 "v684" V685 "v685" V686 "v686" V687 "v687"
    V688 "v688" V689 "v689" V690 "v690" V691 "v691" V692 "v692" V693 "v693" V694 "v694" V695 "v695"
    V696 "v696" V697 "v697" V698 "v698" V699 "v699" V700 "v700" V701 "v701" V702 "v702" V703 "v703"
    V704 "v704" V705 "v705" V706 "v706" V707 "v707" V708 "v708" V709 "v709" V710 "v710" V711 "v711"
    V712 "v712" V713 "v713" V714 "v714" V715 "v715" V716 "v716" V717 "v717" V718 "v718" V719 "v719"
    V720 "v720" V721 "v721" V722 "v722" V723 "v723" V724 "v724" V725 "v725" V726 "v726" V727 "v727"
    V728 "v728" V729 "v729" V730 "v730" V731 "v731" V732 "v732" V733 "v733" V734 "v734" V735 "v735"
    V736 "v736" V737 "v737" V738 "v738" V739 "v739" V740 "v740" V741 "v741" V742 "v742" V743 "v743"
    V744 "v744" V745 "v745" V746 "v746" V747 "v747" V748 "v748" V749 "v749" V750 "v750" V751 "v751"
    V752 "v752" V753 "v753" V754 "v754" V755 "v755" V756 "v756" V757 "v757" V758 "v758" V759 "v759"
    V760 "v760" V761 "v761" V762 "v762" V763 "v763" V764 "v764" V765 "v765" V766 "v766" V767 "v767"
    V768 "v768" V769 "v769" V770 "v770" V771 "v771" V772 "v772" V773 "v773" V774 "v774" V775 "v775"
    V776 "v776" V777 "v777" V778 "v778" V779 "v779" V780 "v780" V781 "v781" V782 "v782" V783 "v783"
    V784 "v784" V785 "v785" V786 "v786" V787 "v787" V788 "v788" V789 "v789" V790 "v790" V791 "v791"
    V792 "v792" V793 "v793" V794 "v794" V795 "v795" V796 "v796" V797 "v797" V798 "v798" V799 "v799"
}

#[test]
fn an_enum_of_eight_hundred_variants_compiles_and_names_each_one() {
    assert_eq!(Wide::VARIANTS.len(), 800);
    assert_eq!(Wide::V0.name(), "v0");
    assert_eq!(Wide::V799.name(), "v799");
    assert_eq!("v799".parse::<Wide>(), Ok(Wide::V799));
}

#[derive(Named, VariantArray, Clone, Copy)]
#[accessor(r#glyph: core::primitive::u8)]
#[accessor(rank: std::primitive::u16)]
#[accessor(wide: ::core::primitive::i64)]
enum Spelled {
    #[name("one")]
    #[value(glyph = 1)]
    #[value(r#rank = 2)]
    #[value(wide = 3)]
    One,
}

#[test]
fn accessors_accept_primitive_paths_and_raw_names() {
    assert_eq!((Spelled::One.glyph(), Spelled::One.rank(), Spelled::One.wide()), (1, 2, 3));
}

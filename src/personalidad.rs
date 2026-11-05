//! La personalidad del sistema solar: lo que dice cada planeta cuando uno llega, lo que opina
//! GLADAS si uno se queda sin hacer nada y las frases del menu. Humor chapin y de internet.

/// Lo que dice cada planeta al acercarse (va rotando entre visitas).
pub fn saludos(archivo: &str) -> &'static [&'static str] {
    match archivo {
        "sol" => &["Sol: ¿Y ese bloqueador? Yo soy factor un millón. Así que usted verá.", "Sol: No me vea directo. Mis lentes son de marca y aun así.", "Sol: Todos giran alrededor mío. Y no es que me lo tenga creído: es gravedad."],
        "mercurio" => &["Mercurio: ¡Rápido, rápido, que voy tarde! Doy la vuelta en 88 días y aún así no llego a nada.", "Mercurio: Dicen que cuando estoy retrógrado todo sale mal. Yo solo voy manejando.", "Mercurio: De día 430 grados, de noche -180. Como el aire acondicionado de la U."],
        "venus" => &["Venus: Giro al revés. No es que esté mal: es que todos ustedes están al revés.", "Venus: Mi día dura más que mi año. Tengo reuniones que también.", "Venus: Soy el más caliente aunque no sea el más cercano. Es actitud."],
        "tierra" => &["Tierra: Disculpe el desorden. Tengo humanos.", "Tierra: Si viene por el wifi, está malo. Si viene por los shucos, en la esquina.", "Tierra: Usted salió de aquí y ni escribe. Su mamá pregunta."],
        "luna" => &["Luna: Sí fueron. No fueron. Yo no doy declaraciones sin mi abogado.", "Luna: Siempre le enseño el mismo lado a la Tierra. El otro es para los amigos.", "Luna: Silencio, que estamos grabando."],
        "marte" => &["Marte: Hay un carrito dando vueltas por aquí. Yo creo que le gusto.", "Marte: Tengo el volcán más grande del sistema solar. No es por presumir. Sí es.", "Marte: Sí, soy rojo. No, no estoy enojado. Bueno, un poquito."],
        "jupiter" => &["Júpiter: Esta mancha es una tormenta de 350 años. Yo también tengo mis días.", "Júpiter: Tengo 95 lunas. Ya ni les aprendo el nombre.", "Júpiter: Si me hubiera puesto a dieta, no sería planeta gigante. Ni modo."],
        "urano" => &["Urano: Estoy acostado. Tengo permiso. Lleva 98 grados de permiso.", "Urano: No haga el chiste de mi nombre. Doña Chepa ya lo hizo.", "Urano: Si necesita el baño, son cinco quetzales. Sin papel."],
        "hada" => &["Planeta Hada: ¡Bienvenida, bienvenida! Aquí todo brilla. Hasta las deudas.", "Planeta Hada: Si saca la ocarina, yo salgo. Es contrato.", "Planeta Hada: No soy real. Y aun así tengo más seguidores que Plutón."],
        "chapin" => &["Planeta Chapín: ¡Púchica! Pase adelante. ¿Ya comió?", "Planeta Chapín: Hay campo, hay campo. Siempre hay campo.", "Planeta Chapín: Ahorita le ayudo. Ahorita es entre hoy y marzo."],
        "planeta_vegetta" => &["Planeta Vegetta: ¡Qué pasa, guapísimo! Sí, soy un cubo. Y es lo mejor que me ha pasado.", "Planeta Vegetta: Aquí se pica, se construye y se pierde contra Willy.", "Planeta Vegetta: Redondo es para los que no tienen imaginación."],
        "saturno_ritmo" => &["Saturno: Estos anillos los pagué yo. Bueno, el juez dirá.", "Saturno: Súbale. Súbale más. Esta es la buena.", "Saturno: No me hable de mi ex. Me dejó sin la mitad de los anillos."],
        "neptuno" => &["Neptuno: ¿Juega cartas? Aquí soplan 2,000 km/h. Agarre las suyas.", "Neptuno: Llueven diamantes y aun así no me alcanza para nada.", "Neptuno: +4. Era broma. ¿O no?"],
        "pluton" => &["Plutón: ¿Usted también viene a decirme que no soy planeta?", "Plutón: Tengo un corazón enorme en la superficie. Nadie pregunta.", "Plutón: Soy planeta. Soy planeta. Soy planeta. Si lo digo tres veces, cuenta."],
        _ => &[],
    }
}

/// Lo que dice GLADAS si uno se queda quieto un buen rato.
pub const OCIO: [&str; 8] = [
    "¿Sigue ahí? Le pregunto porque el universo se expande y usted no.",
    "Yo aquí calculando trayectorias y usted viendo el paisaje. Está bien. Está bonito.",
    "Modo AFK detectado. Le voy a poner el estado en «ocupado».",
    "Si se durmió, no se preocupe: yo manejo. Mentira, no sé manejar. Despierte.",
    "Las notas no se van a recoger solas. Lo verifiqué. Dos veces.",
    "Aprovecho para decirle que su nave tiene garantía hasta Marte. Ya pasamos Marte.",
    "Llevo un rato sin hablar y ya me sentía raro. ¿Cómo está la familia?",
    "Pruebe a tocar algo. Lo que sea. Menos el Sol.",
];

/// Las frases que salen en el menu, una distinta cada vez. Sin acentos: van con la fuente del titulo.
pub const FRASES_MENU: [&str; 20] = [
    "HAY CAMPO!", "AHORITA LE AYUDO", "PLUTON ES PLANETA", "SIN CAMARA 3D DE RAYLIB", "100% SOFTWARE",
    "TODO EN VOXELES", "NO TOQUE EL SOL", "CON TODO, SIEMPRE", "+4 Y SIN RENCORES", "SHADERS HECHOS A MANO",
    "SALIO DE LA ROOSEVELT", "ENTREGA 23:59", "AURA +67", "GLADAS LE VIGILA", "TIENE 200 PUNTOS?",
    "PUCHICA!", "NIVEL 40: BANANINI", "EL PERRO ESTA BIEN", "Y EL WIFI?", "CAFE Y ANSIEDAD",
];

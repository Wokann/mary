/* Shared ordered callable IDs. Select one MARY_* target in the .mary.c source. */
#if !(defined(MARY_FOMT_US) || defined(MARY_MFOMT_US) || defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))
#error Select exactly one MARY_* target
#endif
#if (defined(MARY_FOMT_US) && (defined(MARY_MFOMT_US) || defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))) || (defined(MARY_MFOMT_US) && (defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))) || (defined(MARY_FOMT_JP) && defined(MARY_MFOMT_JP))
#error Select exactly one MARY_* target
#endif
#if defined(MARY_FOMT_US)
#define MARY_FOMT
#define MARY_US
#elif defined(MARY_MFOMT_US)
#define MARY_MFOMT
#define MARY_US
#elif defined(MARY_FOMT_JP)
#define MARY_FOMT
#define MARY_JP
#elif defined(MARY_MFOMT_JP)
#define MARY_MFOMT
#define MARY_JP
#endif
mary_callable_table
{
    NULL,
    NULL,
    SetEntityPosition,
    GetEntityX,
    GetEntityY,
    SetEntityFacing,
    GetEntityFacing,
    Proc007,
    Proc008,
    Proc009,
    Proc00A,
    Proc00B,
    Proc00C,
    SetEntityAnim,
    NULL,
    Proc00F,
    Proc010,
    Proc011,
    Proc012,
    GetOppositeFacing,
    GetEntityLocation,
    Proc015,
    Proc016,
    Proc017,
    Proc018,
    PlayBGM,
    StopBGM,
    PlaySong,
    StopAllSongs,
    FadeOutBGM,
    NULL,
    TalkOpen,
#if defined(MARY_MFOMT)
    Proc020,
#endif
    TalkClose,
    TalkMessage,
    TalkMessageSlow,
    Proc024,
    Func025,
    NULL,
    Func027,
    TalkChoice2,
    TalkChoice3,
    TalkChoice4,
    TalkChoice5,
    TalkChoice6,
    Proc02D,
    Proc02E,
    Proc02F,
    Proc030,
    Proc031,
    Proc032,
#if defined(MARY_FOMT)
    NULL,
#elif defined(MARY_MFOMT)
    Proc033,
#endif
    Proc034,
    Proc035,
    Proc036,
    Proc037,
    Proc038,
    Proc039,
    Proc03A,
    Proc03B,
    NULL,
    Func03D,
    Func03E,
    Proc03F,
    Func040,
    Func041,
    Func042,
    Func043,
    Func044,
    Func045,
    Proc046,
    Proc047,
    Proc048,
    Proc049,
    Proc04A,
    Proc04B,
    Func04C,
    Func04D,
    Proc04E,
    Func04F,
    Func050,
    Proc051,
    Proc052,
    Func053,
    Func054,
    Proc055,
    Func056,
    Func057,
#if defined(MARY_FOMT)
    NULL,
#elif defined(MARY_MFOMT)
    Func058,
#endif
    Func059,
    Func05A,
    Proc05B,
    Proc05C,
    Func05D,
    Func05E,
    NULL,
    Func060,
    Proc061,
    Proc062,
    Proc063,
    Func064,
    Proc065,
    Proc066,
    Proc067,
    Proc068,
    Func069,
    Proc06A,
    Proc06B,
    Proc06C,
    Func06D,
    Func06E,
    Func06F,
    NULL,
    Func071,
    NULL,
    Proc073,
    Func074,
    Proc075,
    NULL,
    Proc077,
#if defined(MARY_MFOMT)
    Func078,
    Proc079,
#endif
    Func07A,
    Func07B,
    Proc07C,
    Proc07D,
    Func07E,
    Func07F,
    Proc080,
    Proc081,
    Func082,
    Proc083,
    Func084,
    Func085,
    Func086,
    Proc087,
    Func088,
    Func089,
    Proc08A,
    NULL,
    Proc08C,
    Proc08D,
    Proc08E,
    Proc08F,
    Proc090,
    Proc091,
    Proc092,
    Proc093,
    Proc094,
    Proc095,
    Proc096,
    Proc097,
    Proc098,
    Proc099,
    Proc09A,
    Proc09B,
    Proc09C,
    Proc09D,
    Proc09E,
    Proc09F,
    Proc0A0,
    Proc0A1,
    Proc0A2,
    Proc0A3,
    Proc0A4,
    Func0A5,
    Proc0A6,
#if defined(MARY_FOMT)
    Proc0A7,
#elif defined(MARY_MFOMT)
    NULL,
#endif
    Proc0A8,
    Proc0A9,
    Func0AA,
    Proc0AB,
    Proc0AC,
    Proc0AD,
    Proc0AE,
    Proc0AF,
    Proc0B0,
    Proc0B1,
    Proc0B2,
    Func0B3,
    Func0B4,
    Func0B5,
    Proc0B6,
    Func0B7,
    Proc0B8,
    Func0B9,
    Func0BA,
    Proc0BB,
    Proc0BC,
    Func0BD,
    Func0BE,
    Func0BF,
    Func0C0,
    Func0C1,
    Proc0C2,
    Proc0C3,
    Func0C4,
    Func0C5,
    Func0C6,
    Proc0C7,
    Proc0C8,
    Func0C9,
    Proc0CA,
    Proc0CB,
    Proc0CC,
    Proc0CD,
    Func0CE,
    Func0CF,
    Func0D0,
    Func0D1,
    NULL,
    Proc0D3,
    Proc0D4,
    Func0D5,
    Func0D6,
    Func0D7,
    Func0D8,
    Func0D9,
    Func0DA,
    Proc0DB,
    Proc0DC,
    Func0DD,
    Proc0DE,
    Proc0DF,
    Func0E0,
    Func0E1,
    Func0E2,
    Proc0E3,
    Proc0E4,
    Proc0E5,
    Func0E6,
    Func0E7,
    Func0E8,
    Func0E9,
    Func0EA,
    Func0EB,
    Func0EC,
    Func0ED,
    NULL,
    Func0EF,
    Proc0F0,
    Proc0F1,
    Proc0F2,
    Proc0F3,
    Func0F4,
    Func0F5,
    Func0F6,
    Proc0F7,
    Func0F8,
    Func0F9,
    Proc0FA,
    Proc0FB,
    Func0FC,
    Func0FD,
    Func0FE,
    Proc0FF,
    Proc100,
    Proc101,
    Func102,
    Proc103,
    Proc104,
    GetAnimalName,
    Func106,
    HasAnimalBeenTalkedTo,
    SetAnimalTalkedTo,
    Proc109,
    Func10A,
    GetAnimalAge,
    Func10C,
    Func10D,
    NULL,
    Func10F,
    Func110,
    Func111,
    Func112,
    Proc113,
    Proc114,
    Proc115,
    Proc116,
    Func117,
    Func118,
    Func119,
    Func11A,
    Proc11B,
    Proc11C,
    Func11D,
    Proc11E,
    Func11F,
    Func120,
    Func121,
    Func122,
#if defined(MARY_MFOMT)
    NULL,
#endif
    Func124,
    Func125,
    Func126,
    Func127,
    Func128,
    Proc129,
    Proc12A,
    Proc12B,
    Proc12C,
    Proc12D,
    Proc12E,
    Proc12F,
    Func130,
    Proc131,
    Proc132,
    Proc133,
    Proc134,
    Proc135,
    Proc136,
    Func137,
    Proc138,
    Proc139,
    Func13A,
    Func13B,
    Func13C,
    Proc13D,
    Proc13E,
    Func13F,
    NULL,
    NULL,
    Proc142,
    Proc143,
    Proc144,
    Proc145,
    Proc146,
    Proc147,
    Proc148,
    Func149,
    Proc14A,
};

void FadeOutBGM(void);
int Func025(const char *arg_1, const char *arg_2, const char *arg_3);
int Func027(const char *arg_1, const char *arg_2, const char *arg_3, const char *arg_4, const char *arg_5);
int Func03D(int arg_1, int arg_2);
int Func03E(int arg_1);
int Func040(void);
int Func041(void);
int Func042(void);
int Func043(void);
int Func044(void);
int Func045(void);
int Func04C(void);
int Func04D(void);
int Func04F(void);
int Func050(void);
int Func053(int arg_1);
int Func054(int arg_1);
int Func056(void);
int Func057(void);
int Func058(int arg_1, int arg_2);
int Func059(int arg_1, int arg_2);
int Func05A(int arg_1, int arg_2);
int Func05D(void);
int Func05E(int arg_1);
int Func060(int arg_1);
int Func064(void);
int Func069(void);
int Func06D(void);
int Func06E(void);
int Func06F(void);
int Func071(int arg_1);
int Func074(void);
int Func078(void);
int Func07A(int arg_1);
int Func07B(int arg_1);
int Func07E(int arg_1);
int Func07F(int arg_1);
int Func082(int arg_1);
int Func084(int arg_1);
int Func085(int arg_1);
int Func086(int arg_1);
int Func088(int arg_1);
int Func089(int arg_1);
int Func0A5(void);
int Func0AA(int arg_1);
int Func0B3(void);
int Func0B4(int arg_1);
int Func0B5(void);
int Func0B7(int arg_1);
int Func0B9(void);
int Func0BA(int arg_1);
int Func0BD(int arg_1);
int Func0BE(void);
int Func0BF(int arg_1);
int Func0C0(int arg_1);
int Func0C1(int arg_1);
int Func0C4(void);
int Func0C5(int arg_1);
int Func0C6(int arg_1);
int Func0C9(void);
int Func0CE(int arg_1);
int Func0CF(int arg_1);
int Func0D0(int arg_1, int arg_2);
int Func0D1(int arg_1);
int Func0D5(int arg_1);
int Func0D6(int arg_1);
int Func0D7(int arg_1);
int Func0D8(int arg_1);
int Func0D9(void);
int Func0DA(int arg_1);
int Func0DD(int arg_1);
int Func0E0(void);
int Func0E1(void);
int Func0E2(void);
int Func0E6(void);
int Func0E7(void);
int Func0E8(void);
int Func0E9(void);
int Func0EA(void);
int Func0EB(void);
int Func0EC(void);
int Func0ED(void);
int Func0EF(void);
int Func0F4(void);
int Func0F5(void);
int Func0F6(void);
int Func0F8(int arg_1);
int Func0F9(int arg_1);
int Func0FC(void);
int Func0FD(void);
int Func0FE(const char *arg_1);
int Func102(int arg_1, int arg_2);
int Func106(void);
int Func10A(int arg_1, int arg_2);
int Func10C(int arg_1, int arg_2);
int Func10D(int arg_1, int arg_2);
int Func10F(int arg_1, int arg_2);
int Func110(int arg_1, int arg_2);
int Func111(int arg_1);
int Func112(int arg_1, int arg_2);
int Func117(int arg_1);
int Func118(void);
int Func119(void);
int Func11A(void);
int Func11D(int arg_1);
int Func11F(void);
int Func120(void);
int Func121(void);
int Func122(void);
int Func124(void);
int Func125(void);
int Func126(void);
int Func127(void);
int Func128(void);
int Func130(int arg_1);
int Func137(void);
int Func13A(int arg_1);
int Func13B(int arg_1);
int Func13C(int arg_1);
int Func13F(int arg_1);
int Func149(void);
int GetAnimalAge(int arg_1, int arg_2);
void GetAnimalName(int arg_1, int arg_2, int arg_3);
int GetEntityFacing(int arg_1);
int GetEntityLocation(int arg_1);
int GetEntityX(int arg_1);
int GetEntityY(int arg_1);
int GetOppositeFacing(int arg_1);
int HasAnimalBeenTalkedTo(int arg_1, int arg_2);
void PlayBGM(int arg_1, int arg_2);
void PlaySong(int arg_1, int arg_2);
void Proc007(int arg_1, int arg_2);
void Proc008(int arg_1, int arg_2, int arg_3);
void Proc009(int arg_1, int arg_2, int arg_3);
void Proc00A(int arg_1, int arg_2, int arg_3);
void Proc00B(int arg_1, int arg_2, int arg_3);
void Proc00C(int arg_1);
void Proc00F(int arg_1);
void Proc010(int arg_1, int arg_2);
void Proc011(int arg_1, int arg_2, int arg_3);
void Proc012(int arg_1);
void Proc015(int arg_1, int arg_2, int arg_3);
void Proc016(int arg_1, int arg_2, int arg_3);
void Proc017(int arg_1, int arg_2, int arg_3);
void Proc018(void);
void Proc020(void);
void Proc024(const char *arg_1);
void Proc02D(int arg_1);
void Proc02E(const char *arg_1);
void Proc02F(void);
void Proc030(int arg_1);
void Proc031(void);
void Proc032(int arg_1);
void Proc033(void);
void Proc034(int arg_1, int arg_2);
void Proc035(int arg_1, int arg_2);
void Proc036(int arg_1, int arg_2);
void Proc037(int arg_1);
void Proc038(int arg_1);
void Proc039(int arg_1, int arg_2);
void Proc03A(int arg_1, int arg_2, int arg_3);
void Proc03B(int arg_1, const char *arg_2);
void Proc03F(int arg_1, int arg_2);
void Proc046(void);
void Proc047(void);
void Proc048(int arg_1);
void Proc049(int arg_1);
void Proc04A(int arg_1);
void Proc04B(int arg_1);
void Proc04E(void);
void Proc051(int arg_1, int arg_2);
void Proc052(void);
void Proc055(int arg_1);
void Proc05B(int arg_1);
void Proc05C(int arg_1, int arg_2);
void Proc061(int arg_1);
void Proc062(void);
void Proc063(void);
void Proc065(void);
void Proc066(void);
void Proc067(void);
void Proc068(void);
void Proc06A(void);
void Proc06B(void);
void Proc06C(void);
void Proc073(void);
void Proc075(void);
void Proc077(int arg_1);
void Proc079(int arg_1);
void Proc07C(int arg_1);
void Proc07D(int arg_1);
void Proc080(int arg_1, int arg_2);
void Proc081(int arg_1, int arg_2);
void Proc083(int arg_1);
void Proc087(int arg_1);
void Proc08A(int arg_1, int arg_2);
void Proc08C(int arg_1, int arg_2);
void Proc08D(int arg_1);
void Proc08E(void);
void Proc08F(int arg_1);
void Proc090(void);
void Proc091(void);
void Proc092(void);
void Proc093(void);
void Proc094(void);
void Proc095(void);
void Proc096(void);
void Proc097(void);
void Proc098(void);
void Proc099(void);
void Proc09A(void);
void Proc09B(int arg_1);
void Proc09C(void);
void Proc09D(void);
void Proc09E(void);
void Proc09F(void);
void Proc0A0(void);
void Proc0A1(void);
void Proc0A2(void);
void Proc0A3(void);
void Proc0A4(void);
void Proc0A6(int arg_1, int arg_2);
void Proc0A7(void);
void Proc0A8(void);
void Proc0A9(int arg_1);
void Proc0AB(void);
void Proc0AC(int arg_1);
void Proc0AD(void);
void Proc0AE(void);
void Proc0AF(void);
void Proc0B0(void);
void Proc0B1(void);
void Proc0B2(void);
void Proc0B6(int arg_1);
void Proc0B8(int arg_1);
void Proc0BB(int arg_1);
void Proc0BC(int arg_1);
void Proc0C2(int arg_1);
void Proc0C3(void);
void Proc0C7(void);
void Proc0C8(void);
void Proc0CA(int arg_1);
void Proc0CB(int arg_1);
void Proc0CC(void);
void Proc0CD(void);
void Proc0D3(int arg_1, int arg_2, int arg_3);
void Proc0D4(int arg_1);
void Proc0DB(int arg_1);
void Proc0DC(void);
void Proc0DE(void);
void Proc0DF(void);
void Proc0E3(int arg_1);
void Proc0E4(void);
void Proc0E5(void);
void Proc0F0(int arg_1);
void Proc0F1(int arg_1);
void Proc0F2(void);
void Proc0F3(void);
void Proc0F7(int arg_1, int arg_2);
void Proc0FA(int arg_1);
void Proc0FB(int arg_1);
void Proc0FF(int arg_1);
void Proc100(void);
void Proc101(void);
void Proc103(int arg_1, int arg_2, int arg_3, int arg_4, int arg_5);
void Proc104(int arg_1, int arg_2);
void Proc109(int arg_1, int arg_2, int arg_3);
void Proc113(void);
void Proc114(void);
void Proc115(void);
void Proc116(void);
void Proc11B(int arg_1, int arg_2);
void Proc11C(int arg_1);
void Proc11E(int arg_1);
void Proc129(const char *arg_1);
void Proc12A(void);
void Proc12B(int arg_1, int arg_2, int arg_3);
void Proc12C(void);
void Proc12D(void);
void Proc12E(int arg_1);
void Proc12F(void);
void Proc131(void);
void Proc132(int arg_1, int arg_2, int arg_3, int arg_4);
void Proc133(void);
void Proc134(void);
void Proc135(void);
void Proc136(void);
void Proc138(int arg_1, int arg_2);
void Proc139(void);
void Proc13D(int arg_1, int arg_2, int arg_3, int arg_4, int arg_5);
void Proc13E(int arg_1);
void Proc142(int arg_1, int arg_2, int arg_3, int arg_4, int arg_5);
void Proc143(int arg_1, int arg_2, int arg_3);
void Proc144(int arg_1, int arg_2, int arg_3);
void Proc145(int arg_1);
void Proc146(int arg_1);
void Proc147(int arg_1, int arg_2, int arg_3);
void Proc148(int arg_1, int arg_2);
void Proc14A(void);
void SetAnimalTalkedTo(int arg_1, int arg_2);
void SetEntityAnim(int arg_1, int arg_2);
void SetEntityFacing(int arg_1, int arg_2);
void SetEntityPosition(int arg_1, int arg_2, int arg_3, int arg_4);
void StopAllSongs(void);
void StopBGM(void);
int TalkChoice2(const char *arg_1, const char *arg_2);
int TalkChoice3(const char *arg_1, const char *arg_2, const char *arg_3);
int TalkChoice4(const char *arg_1, const char *arg_2, const char *arg_3, const char *arg_4);
int TalkChoice5(const char *arg_1, const char *arg_2, const char *arg_3, const char *arg_4, const char *arg_5);
int TalkChoice6(const char *arg_1, const char *arg_2, const char *arg_3, const char *arg_4, const char *arg_5, const char *arg_6);
void TalkClose(void);
void TalkMessage(const char *arg_1);
void TalkMessageSlow(const char *arg_1);
void TalkOpen(void);

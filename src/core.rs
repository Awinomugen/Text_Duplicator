use eframe::{
    egui::{self, FontDefinitions, FontFamily,FontData,Ui,RichText,Color32,ScrollArea}
    ,epi};

use std::fs::File;
use std::io::Write;

use std::path::{Path};

use crate::parser::parse_sequence;

#[derive(Default)]
pub struct AppBody{
    //入力
    text_in:String,
    //複製数
    num:String,
    //入力に関するエラー
    err_in:String,

    //置換リスト
    rep:[(String,String);3],

    //出力
    text_out:String,
    //出力ファイル名
    out_fname:String,
    //出力に関するエラー
    err_out:String,

    //追加情報を表示するタイマー
    out_info_timer:u32,

    //直接ファイルに書き出すフラグ
    direct_file:bool
}

//GUIの構成を実装
impl epi::App for AppBody{
    //ウィンドウのタイトル
    fn name(&self)->&str{
        "Text_Duplicator"
    }

    //起動時に呼ぶやつ
    fn setup(
        &mut self,
        ctx:&egui::Context,
        _frame:&epi::Frame,
        _storage:Option<&dyn epi::Storage>)
    {
        //日本語フォントの設定
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "jp_font".to_owned(),
            FontData::from_owned(include_bytes!("../font/NotoSansJP-Medium.otf").to_vec()),
        );
        
        fonts.families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "jp_font".to_owned());
            
        ctx.set_fonts(fonts);

        //設定の復元(もしくはデフォルト)
        #[cfg(feature = "persistence")]
        if let Some(storage)=_storage{
            *self=epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        //出力ファイル名のデフォルト
        self.out_fname=String::from("out.txt");
        //デフォルトでは直接書き出さない
        self.direct_file=false;
    }

    //終了時に呼ぶやつ
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        //設定の保存
        epi::set_value(storage, epi::APP_KEY, self);
    }


    //常に実行されるやつ
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        //メイン画面
        //サブ画面を出す予定はない
        egui::CentralPanel::default().show(ctx, |ui:&mut Ui| {
            ui.horizontal_top(|ui:&mut Ui| {
                //左サイド
                //入力関係
                ui.vertical(|ui:&mut Ui| {
                    ui.label(RichText::new("入力").color(Color32::RED).size(20.0));
                    //スクロール式にして、大きくならないように
                    ScrollArea::vertical()
                    .id_source("in_scr")
                    .max_height(80.0).show(ui,|ui:&mut Ui|{
                        ui.text_edit_multiline(&mut self.text_in);
                    });
                    ui.label(RichText::new("複製数").color(Color32::RED).size(20.0));
                    ui.text_edit_singleline(&mut self.num);

                    ui.horizontal(|ui:&mut Ui|{
                        if self.direct_file {
                            
                            //大きいテキストを出力した後戻すと重いので出力テキストを消しておく
                            self.text_out.clear();
                            if ui.button("出力！").clicked() {
                                if let Err(e)=self.txt_proc_direct(){
                                    //エラー出力
                                    self.err_out=String::from(e);
                                }
                            }
                        }else{
                            if ui.button("変換！").clicked() {
                                if let Err(e)=self.txt_proc(){
                                    //エラー出力
                                    self.err_out=String::from(e);
                                }
                            }
                        }
                        //直接ファイルに出力するか
                        ui.checkbox(&mut self.direct_file, "直接ファイルに出力(高速)");
                    });

                    //エラー表示
                    ui.label(RichText::new(&self.err_in).color(Color32::RED).size(20.0));

                    
                    for i in 0..3{
                        ui.label(RichText::new(format!("置換対象{}",i+1)).color(Color32::RED).size(20.0));
                        ui.text_edit_singleline(&mut self.rep[i].0);
                        ui.label(RichText::new(format!("置換パターン{}",i+1)).color(Color32::RED).size(20.0));
                        ui.text_edit_singleline(&mut self.rep[i].1);
                    }
                });

                //中心の区切り線
                ui.separator();

                //右サイド
                //出力関係
                ui.vertical(|ui:&mut Ui| {
                    //直接出力フラグが立っているならば出力ボックスを出さない
                    if self.direct_file==false{
                        ui.label(RichText::new("出力").color(Color32::BLUE).size(20.0));
                        //スクロール式にして、大きくならないように
                        ScrollArea::vertical()
                        .id_source("out_scr")
                        .max_height(160.0)
                        .auto_shrink([false; 2]).show(ui,|ui:&mut Ui|{
                            ui.text_edit_multiline(&mut self.text_out);
                        });
                    }
                    ui.label(RichText::new("出力ファイル名").color(Color32::BLUE).size(20.0));
                    ui.text_edit_singleline(&mut self.out_fname);
                    //直接出力フラグが立っているならば出力ボタンを出さない
                    if self.direct_file==false{
                        if ui.button("ファイルに出力").clicked() {
                            let r=self.to_file();
                            if let Err(s) = r {
                                //エラー出力
                                self.err_out=String::from(s);
                            }else{
                                self.err_out.clear();
                            }
                        }
                    }
                    
                    //エラー表示
                    ui.label(RichText::new(&self.err_out).color(Color32::RED).size(20.0));

                    //追加情報(ファイル出力の完了のみ)
                    if self.out_info_timer>0{
                        //情報表示タイマーを減算
                        self.out_info_timer-=1;
                        ui.label(RichText::new("出力完了！").color(Color32::BLUE).size(30.0));
                    }
                    
                });
            });

        });
        
        //サイズをでかめに
        ctx.set_pixels_per_point(2.0);
        

    }
}

//自作の(ユーティリティ的)メソッド
impl AppBody{
    //出力をファイルに書き出す
    fn to_file(&mut self)->Result<(),&str>{
        //書き込み用ファイルの作成
        let file=wfile_open(&self.out_fname);

        if let Err(e) = file {
            return Err(e);
        }
        let mut file=file.unwrap();

        //出力をファイルに書き出し
        let r=str_to_file(&mut file,&self.text_out);

        if let Err(e) = r {
            return Err(e);
        }
        //出力完了を知らせる
        self.out_info_timer=50;
        Ok(())
    }

    //文字列処理
    fn txt_proc(&mut self)->Result<(),&str>{
        let num=self.num.parse::<u32>();
        if let Err(_)=num{
            return Err("複製数が不正");
        }
        let num=num.unwrap();

        self.text_out.clear();

        let rep_list=parse_sequence(self.rep.to_vec(), num);

        if let Err(e)=rep_list{
            return Err(e);
        }

        let rep_list=rep_list.unwrap();

        for i in 0..num{
            let mut s=String::from(&self.text_in);
            for j in &rep_list {
                s=s.replace(&j.0, &format!("{}",j.1(i as i32)));
            }
            self.text_out+=&s;
        }
        Ok(())
    }

    //ファイルに直接出力
    fn txt_proc_direct(&mut self)->Result<(),&str>{
        //書き込み用ファイルの作成
        let file=wfile_open(&self.out_fname);

        if let Err(e) = file {
            return Err(e);
        }
        let mut file=file.unwrap();
        
        //複製数を取得
        let num=self.num.parse::<u32>();
        if let Err(_)=num{
            return Err("複製数が不正");
        }
        let num=num.unwrap();

        let rep_list=parse_sequence(self.rep.to_vec(), num);
        if let Err(e)=rep_list{
            return Err(e);
        }

        let rep_list=rep_list.unwrap();
        for i in 0..num{
            let mut s=String::from(&self.text_in);
            for j in &rep_list {
                s=s.replace(&j.0, &format!("{}",j.1(i as i32)));
            }
            if let Err(e)=str_to_file(&mut file,&s){
                return Err(e);
            }
        }
        //出力完了を知らせる
        self.out_info_timer=50;
        Ok(())
    }
}

//書き込み用ファイルを開く
fn wfile_open(fp:&String)->Result<File,&'static str>{
    //ファイル名を取得
    let path=Path::new(fp);
    //ディレクトリを書き込むな
    let fname=path.file_name();
    if let None = fname {
        return Err("不正なファイル名");
    }
    //書き込み用ファイルの作成
    let file=File::create(fname.unwrap());

    if let Err(_) = file {
        return Err("ファイルの作成に失敗");
    }

    Ok(file.unwrap())
}

//ファイルに引数のStringを書き出す
fn str_to_file(f:&mut File,out:&String)->Result<(),&'static str>{
    //出力をファイルに書き出し
    let r=f.write_all(out.as_bytes());

    if let Err(_) = r {
        return Err("ファイルへの書き込みに失敗");
    }
    Ok(())
}

